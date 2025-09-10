use anyhow::{Result, anyhow};
use bytes::Bytes;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, watch};
use tracing::warn;

use crate::camera::{HEIGHT, WIDTH};
use crate::parameters::ParametersController;

const MAX_LOG_ENTRIES: usize = 200;

#[derive(Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Clone)]
pub struct AppState {
    frame_rx: watch::Receiver<Arc<Bytes>>,
    logs_tx: broadcast::Sender<LogEntry>,

    pipeline: gst::Pipeline,
    logs: Arc<RwLock<Vec<LogEntry>>>,

    pub parameters_controller: Arc<RwLock<ParametersController>>,
}

impl AppState {
    fn pipeline_string() -> String {
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
            ! video/x-raw,format=I420,width={WIDTH},height={HEIGHT},framerate=20/1
            ! queue max-size-buffers=1 leaky=downstream
            ! appsink name=sink emit-signals=false sync=false drop=true max-buffers=1 enable-last-sample=false"
        )
    }

    fn create_pipeline() -> Result<gst::Pipeline> {
        let pipeline_string = AppState::pipeline_string();

        gst::init()?;
        let pipeline = gst::parse::launch(&pipeline_string)?
            .downcast::<gst::Pipeline>()
            .map_err(|e| anyhow!("Launching pipeline failed: {}", e.type_().name()))?;

        Ok(pipeline)
    }

    pub async fn new(
        frame_rx: watch::Receiver<Arc<Bytes>>,
        logs: Arc<RwLock<Vec<LogEntry>>>,
        parameters: ParametersController,
    ) -> Result<AppState> {
        let (logs_tx, _logs_rx) = broadcast::channel(MAX_LOG_ENTRIES);

        let app_state = AppState {
            frame_rx,
            logs_tx,
            logs,
            pipeline: AppState::create_pipeline()?,
            parameters_controller: Arc::new(RwLock::new(parameters)),
        };

        app_state.set_awb_enable(true);
        app_state.play_pipeline()?;

        Ok(app_state)
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

    pub fn stop_pipeline(&self) -> Result<()> {
        self.pipeline.set_state(gst::State::Null)?;
        Ok(())
    }

    pub fn play_pipeline(&self) -> Result<()> {
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
