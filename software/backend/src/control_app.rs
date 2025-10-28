use anyhow::{Result, anyhow};
use bytes::Bytes;
use communication::driver::DeviceDriver;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedSemaphorePermit, RwLock, Semaphore, broadcast, watch};
use tracing::warn;

use crate::camera::{PHOTO_HEIGHT, PHOTO_WIDTH, STREAM_HEIGHT, STREAM_WIDTH};
use crate::parameters::{Parameters, ParametersController};

const MAX_LOG_ENTRIES: usize = 200;

#[derive(Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

pub struct AppStateGuard<'a> {
    state: &'a AppState,
    _permit: OwnedSemaphorePermit,
}

#[derive(Clone)]
pub struct AppState {
    frame_rx: watch::Receiver<Arc<Bytes>>,
    logs_tx: broadcast::Sender<LogEntry>,

    sem: Arc<Semaphore>,

    pipeline: gst::Pipeline,
    logs: Arc<RwLock<Vec<LogEntry>>>,

    pub device_driver: Option<Arc<Mutex<DeviceDriver>>>,

    pub parameters_controller: Arc<RwLock<ParametersController>>,
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
            "videotestsrc name=source is-live=true pattern=smpte"
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
    ) -> Result<AppState> {
        let (logs_tx, _logs_rx) = broadcast::channel(MAX_LOG_ENTRIES);

        let app_state = AppState {
            frame_rx,
            logs_tx,
            sem: Arc::new(Semaphore::new(1)),
            logs,
            pipeline: AppState::create_pipeline()?,
            parameters_controller: Arc::new(RwLock::new(parameters)),
            device_driver,
        };

        app_state.set_awb_enable(true);
        app_state.play_pipeline()?;

        Ok(app_state)
    }

    pub async fn with_guard(&self) -> Result<AppStateGuard<'_>> {
        let permit = self.sem.clone().acquire_owned().await?;
        Ok(AppStateGuard {
            state: self,
            _permit: permit,
        })
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

    pub async fn subscribe_to_logs(&self) -> broadcast::Receiver<LogEntry> {
        self.logs_tx.subscribe()
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

    fn pull_nth_sample_data(appsink: &gst_app::AppSink, n: u32) -> Result<Bytes> {
        if n == 0 {
            return Err(anyhow!("n must be greater than 0"));
        }

        let mut sample = appsink.pull_sample()?;

        for _ in 0..n - 1 {
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

    async fn take_photo(&self, parameters: &Parameters) -> Result<Bytes> {
        let photo_pipeline = self.start_photo_pipeline(parameters).await?;
        let appsink = photo_pipeline
            .by_name("sink")
            .unwrap()
            .downcast::<gst_app::AppSink>()
            .unwrap();

        let data = Self::pull_nth_sample_data(&appsink, 5)?;

        photo_pipeline.set_state(gst::State::Null)?;
        self.play_pipeline()?;

        Ok(data)
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
        self.state.take_photo(parameters).await
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
        let logs_tx = self.logs_tx.clone();

        let logs_entry = LogEntry {
            timestamp,
            level,
            message,
        };

        let _ = logs_tx.send(logs_entry.clone());

        tokio::spawn(async move {
            let mut logs = logs.write().await;

            if logs.len() >= MAX_LOG_ENTRIES {
                logs.remove(0);
            }
            logs.push(logs_entry);
        });
    }
}
