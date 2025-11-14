use anyhow::{Result, anyhow};
use bytes::Bytes;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use interface::ws::compute_node::{ComputeNode, ComputeNodeCapabilities};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedSemaphorePermit, RwLock, Semaphore, broadcast, watch};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::camera::{PHOTO_HEIGHT, PHOTO_WIDTH, STREAM_HEIGHT, STREAM_WIDTH};
use crate::compute_node::{ComputeNodeContainer, ComputeNodeContainerExt};
use crate::parameters::ParametersController;
use interface::uart::driver::DeviceDriver;
use interface::ws::{logs::LogEntry, parameters::Parameters};

const MAX_LOG_ENTRIES: usize = 200;

#[derive(Clone)]
pub struct Config {
    pub z_scan_dir: PathBuf,
}

pub struct AppStateGuard<'a> {
    state: &'a AppState,
    _permit: OwnedSemaphorePermit,
}

#[derive(Clone)]
pub enum AppStateEvent {
    Log(LogEntry),
    ComputeNoteUpdate(Vec<ComputeNode>),
}

#[derive(Clone)]
pub struct AppState {
    frame_rx: watch::Receiver<Arc<Bytes>>,
    app_event_tx: broadcast::Sender<AppStateEvent>,

    operation_semaphore: Arc<Semaphore>,
    operation_cancel_token: Arc<RwLock<CancellationToken>>,

    pipeline: gst::Pipeline,
    logs: Arc<RwLock<Vec<LogEntry>>>,

    pub device_driver: Option<Arc<Mutex<DeviceDriver>>>,

    pub parameters_controller: Arc<RwLock<ParametersController>>,

    pub config: Config,

    compute_nodes: ComputeNodeContainer,
}

impl AppState {
    fn video_pipeline_string(width: u32, height: u32) -> String {
        let source_element = if cfg!(target_arch = "aarch64") {
            "libcamerasrc name=source 
                exposure-time-mode=manual
                exposure-time=4000
                awb-enable=true
                awb-mode=daylight
                contrast=1.0
                saturation=1.0"
        } else {
            "videotestsrc name=source is-live=true pattern=gradient"
        };

        format!(
            "{source_element}
            ! video/x-raw,format=I420,width={width},height={height},framerate=20/1
            ! queue max-size-buffers=1 leaky=downstream
            ! appsink name=sink emit-signals=false sync=false drop=true max-buffers=1 enable-last-sample=false"
        )
    }

    fn photo_pipeline_string(width: u32, height: u32, parameters: &Parameters) -> String {
        let source_element = if cfg!(target_arch = "aarch64") {
            #[allow(unused_variables)]
            let exposure_time = parameters.camera_properties.exposure_time.unwrap_or(4000);

            #[allow(unused_variables)]
            let contrast = parameters.camera_properties.contrast.unwrap_or(1.0);

            #[allow(unused_variables)]
            let saturation = parameters.camera_properties.saturation.unwrap_or(1.0);

            format!(
                "libcamerasrc name=source 
                exposure-time-mode=manual
                exposure-time={exposure_time}
                awb-enable=true
                awb-mode=daylight
                contrast={contrast}
                saturation={saturation}"
            )
        } else {
            "videotestsrc name=source is-live=true pattern=smpte".to_string()
        };

        format!(
            "{source_element}
            ! video/x-raw,format=I420,width={width},height={height},framerate=10/1
            ! queue max-size-buffers=1 leaky=downstream
            ! jpegenc
            ! appsink name=sink emit-signals=false sync=false drop=true max-buffers=1 enable-last-sample=false"
        )
    }

    fn create_pipeline() -> Result<gst::Pipeline> {
        let pipeline_string = AppState::video_pipeline_string(STREAM_WIDTH, STREAM_HEIGHT);

        gst::init()?;
        let pipeline = gst::parse::launch(&pipeline_string)?
            .downcast::<gst::Pipeline>()
            .map_err(|e| anyhow!("Launching pipeline failed: {}", e.type_().name()))?;

        Ok(pipeline)
    }

    pub async fn new(
        frame_rx: watch::Receiver<Arc<Bytes>>,
        logs: Arc<RwLock<Vec<LogEntry>>>,
        device_driver: Option<Arc<Mutex<DeviceDriver>>>,
        parameters: ParametersController,
        config: Config,
        compute_nodes: ComputeNodeContainer,
    ) -> Result<AppState> {
        let (app_event_tx, _app_event_rx) = broadcast::channel(MAX_LOG_ENTRIES);

        let app_state = AppState {
            frame_rx,
            app_event_tx,
            operation_cancel_token: Arc::new(RwLock::new(CancellationToken::new())),
            operation_semaphore: Arc::new(Semaphore::new(1)),
            logs,
            pipeline: AppState::create_pipeline()?,
            parameters_controller: Arc::new(RwLock::new(parameters)),
            device_driver,
            config,
            compute_nodes,
        };

        app_state.set_awb_enable(true);
        app_state.play_pipeline()?;

        Ok(app_state)
    }

    pub async fn with_guard(&self) -> Result<AppStateGuard<'_>> {
        let permit = self.operation_semaphore.clone().acquire_owned().await?;
        Ok(AppStateGuard {
            state: self,
            _permit: permit,
        })
    }

    pub async fn cancel_operation(&self) {
        let token = self.operation_cancel_token.read().await;
        token.cancel();
    }

    pub async fn update_from_bytes(&self, bytes: Bytes) -> Result<()> {
        let tf = tempfile::NamedTempFile::new()?;
        std::fs::write(tf.path(), &bytes)?;

        if cfg!(debug_assertions) {
            warn!("Self-update called in debug build, skipping actual update");
            return Ok(());
        } else {
            self_replace::self_replace(tf.path())?;

            const EXIT_DELAY: u64 = 2;
            warn!("Exiting in {EXIT_DELAY} seconds. Restarting needs to be handled externally.");
            tokio::spawn(async {
                tokio::time::sleep(std::time::Duration::from_secs(EXIT_DELAY)).await;
                std::process::exit(0);
            });
        }

        Ok(())
    }

    pub fn subscribe_to_frames(&self) -> watch::Receiver<Arc<Bytes>> {
        self.frame_rx.clone()
    }

    pub fn get_source(&self) -> gst::Element {
        self.pipeline.by_name("source").unwrap()
    }

    pub fn get_sink(&self) -> gst_app::AppSink {
        self.pipeline
            .by_name("sink")
            .unwrap()
            .downcast::<gst_app::AppSink>()
            .unwrap()
    }

    fn set_awb_enable(&self, enable: bool) {
        if cfg!(not(debug_assertions)) {
            let source = self.get_source();
            source.set_property("awb-enable", enable);
        }
    }

    fn stop_pipeline(&self) -> Result<()> {
        self.pipeline.set_state(gst::State::Null)?;
        Ok(())
    }

    fn play_pipeline(&self) -> Result<()> {
        self.pipeline.set_state(gst::State::Playing)?;
        Ok(())
    }

    pub async fn get_logs(&self) -> Vec<LogEntry> {
        let l = self.logs.read().await;
        Vec::from_iter(l.iter().cloned())
    }

    pub async fn subscribe_to_app_events(&self) -> broadcast::Receiver<AppStateEvent> {
        self.app_event_tx.subscribe()
    }

    async fn start_photo_pipeline(&self, parameters: &Parameters) -> Result<gst::Pipeline> {
        self.stop_pipeline()?;

        let pipeline_string =
            AppState::photo_pipeline_string(PHOTO_WIDTH, PHOTO_HEIGHT, parameters);
        let photo_pipeline = gst::parse::launch(&pipeline_string)?
            .downcast::<gst::Pipeline>()
            .map_err(|e| anyhow!("Launching photo pipeline failed: {}", e.type_().name()))?;

        photo_pipeline.set_state(gst::State::Playing)?;
        Ok(photo_pipeline)
    }

    async fn pull_nth_sample_data(&self, appsink: &gst_app::AppSink, n: u32) -> Result<Bytes> {
        if n == 0 {
            return Err(anyhow!("n must be greater than 0"));
        }

        let mut sample = appsink.pull_sample()?;

        for _ in 0..n - 1 {
            if self.operation_cancel_token.read().await.is_cancelled() {
                return Err(anyhow!("Operation cancelled"));
            }

            sample = appsink.pull_sample()?;
        }

        let buffer = sample
            .buffer()
            .ok_or_else(|| anyhow!("Failed to get buffer from sample"))?;

        let map = buffer
            .map_readable()
            .map_err(|_| anyhow!("Failed to map buffer readable"))?;

        Ok(Bytes::copy_from_slice(map.as_slice()))
    }

    pub async fn register_compute_node(&self, capabilities: &ComputeNodeCapabilities) -> String {
        let (node_id, node_list) = self.compute_nodes.register(capabilities).await;

        let msg = AppStateEvent::ComputeNoteUpdate(node_list);
        let _ = self.app_event_tx.send(msg);

        node_id
    }

    pub async fn unregister_compute_node(&self, node_id: &str) {
        let node_list = self.compute_nodes.unregister(node_id).await;

        let msg = AppStateEvent::ComputeNoteUpdate(node_list);
        let _ = self.app_event_tx.send(msg);
    }
}

impl<'a> AppStateGuard<'a> {
    pub fn stop_pipeline(&self) -> Result<()> {
        self.state.stop_pipeline()
    }

    pub fn play_pipeline(&self) -> Result<()> {
        self.state.play_pipeline()
    }

    pub async fn take_photo(&self, parameters: &Parameters) -> Result<Bytes> {
        let photo_pipeline = self.state.start_photo_pipeline(parameters).await?;
        let appsink = photo_pipeline
            .by_name("sink")
            .unwrap()
            .downcast::<gst_app::AppSink>()
            .unwrap();

        let ret = self.state.pull_nth_sample_data(&appsink, 5).await;

        photo_pipeline.set_state(gst::State::Null)?;
        self.play_pipeline()?;

        ret
    }

    async fn z_scan_internal(
        &self,
        photo_pipeline: &gst::Pipeline,
        relative_start_pos: i32,
        relative_stop_pos: i32,
        delta_steps: u32,
    ) -> Result<Vec<Bytes>> {
        let device_driver = self.state.device_driver.as_ref().unwrap();

        let appsink = photo_pipeline
            .by_name("sink")
            .unwrap()
            .downcast::<gst_app::AppSink>()
            .unwrap();

        let mut device_driver = device_driver.lock().await;

        device_driver
            .stage_move_steps::<String>(relative_start_pos, 1000)
            .await?;

        let mut current_pos = relative_start_pos;
        let delta = if relative_start_pos < relative_stop_pos {
            delta_steps as i32
        } else {
            -(delta_steps as i32)
        };

        let mut frames = Vec::new();
        loop {
            info!("Taking photo at Z position: {}", current_pos);
            let photo_data = self.state.pull_nth_sample_data(&appsink, 5).await?;
            frames.push(photo_data);

            device_driver
                .stage_move_steps::<String>(delta, 2000)
                .await?;

            current_pos += delta;
            if (delta > 0 && current_pos > relative_stop_pos)
                || (delta < 0 && current_pos < relative_stop_pos)
            {
                break;
            }
        }

        // Return to initial position
        device_driver
            .stage_move_steps::<String>(-current_pos, 1000)
            .await?;

        Ok(frames)
    }

    pub async fn z_scan(
        &self,
        parameters: &Parameters,
        relative_start_pos: i32,
        relative_stop_pos: i32,
        delta_steps: u32,
    ) -> Result<Vec<Bytes>> {
        if delta_steps == 0 {
            return Err(anyhow!("delta_steps must be greater than 0"));
        }

        if self.state.device_driver.is_none() {
            return Err(anyhow!("Device driver not available for Z-scan"));
        }

        let photo_pipeline = self.state.start_photo_pipeline(parameters).await?;

        let ret = self
            .z_scan_internal(
                &photo_pipeline,
                relative_start_pos,
                relative_stop_pos,
                delta_steps,
            )
            .await;

        photo_pipeline.set_state(gst::State::Null)?;
        self.play_pipeline()?;

        ret
    }
}

#[derive(Default)]
struct EventFields {
    message: Option<String>,
    kv: Vec<(String, String)>,
}

impl tracing_subscriber::field::Visit for EventFields {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let v = format!("{value:?}");
        if field.name() == "message" {
            self.message = Some(v.clone());
        }
        self.kv.push((field.name().to_string(), v));
    }
}

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for AppState {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut event_fields = EventFields::default();
        event.record(&mut event_fields);

        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let level = format!("{}", event.metadata().level());
        let message = event_fields.message.unwrap_or_default();

        let logs = self.logs.clone();
        let app_event_tx = self.app_event_tx.clone();

        let logs_entry = LogEntry {
            timestamp,
            level,
            message,
        };

        let _ = app_event_tx.send(AppStateEvent::Log(logs_entry.clone()));

        tokio::spawn(async move {
            let mut logs = logs.write().await;

            if logs.len() >= MAX_LOG_ENTRIES {
                logs.remove(0);
            }
            logs.push(logs_entry);
        });
    }
}
