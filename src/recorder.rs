use chrono::Local;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecorderError {
    #[error("GStreamer Fehler: {0}")]
    GStreamerError(String),
    #[error("Pipeline konnte nicht erstellt werden: {0}")]
    PipelineError(String),
    #[error("Frame konnte nicht geschrieben werden: {0}")]
    WriteError(String),
    #[error("Metadaten konnten nicht gespeichert werden: {0}")]
    MetadataError(String),
    #[error("IO Fehler: {0}")]
    IOError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, RecorderError>;

#[derive(Serialize, Deserialize, Debug)]
pub struct RecordingMetadata {
    pub camera_id: u32,
    pub timestamp: String,
    pub duration_secs: f64,
    pub fps: f64,
    pub width: i32,
    pub height: i32,
    pub filename: String,
}

pub struct VideoRecorder {
    pipeline: gst::Pipeline,
    appsrc: gst_app::AppSrc,
    camera_id: u32,
    start_time: std::time::Instant,
    frame_count: Arc<Mutex<u64>>,
    fps: f64,
    width: i32,
    height: i32,
    output_path: PathBuf,
    is_recording: Arc<Mutex<bool>>,
}

impl VideoRecorder {
    pub fn new(
        camera_id: u32,
        width: i32,
        height: i32,
        fps: f64,
        output_dir: &Path,
    ) -> Result<Self> {
        // Initialisiere GStreamer
        gst::init().map_err(|e| RecorderError::GStreamerError(e.to_string()))?;

        fs::create_dir_all(output_dir)?;

        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("camera_{}__{}.mp4", camera_id, timestamp);
        let output_path = output_dir.join(&filename);

        // Erstelle GStreamer Pipeline
        // appsrc ! videoconvert ! x264enc ! mp4mux ! filesink
        let pipeline_str = format!(
            "appsrc name=src ! videoconvert ! x264enc speed-preset=fast tune=zerolatency ! mp4mux ! filesink location={}",
            output_path.to_str().unwrap()
        );

        let pipeline = gst::parse::launch(&pipeline_str)
            .map_err(|e| RecorderError::PipelineError(e.to_string()))?
            .downcast::<gst::Pipeline>()
            .map_err(|_| RecorderError::PipelineError("Kein Pipeline Element".to_string()))?;

        let appsrc = pipeline
            .by_name("src")
            .ok_or_else(|| RecorderError::PipelineError("appsrc nicht gefunden".to_string()))?
            .downcast::<gst_app::AppSrc>()
            .map_err(|_| RecorderError::PipelineError("Kein AppSrc Element".to_string()))?;

        // Konfiguriere AppSrc
        let caps = gst::Caps::builder("video/x-raw")
            .field("format", "RGB")
            .field("width", width)
            .field("height", height)
            .field("framerate", gst::Fraction::new(fps as i32, 1))
            .build();

        appsrc.set_caps(Some(&caps));
        appsrc.set_property("format", gst::Format::Time);

        // Starte Pipeline
        pipeline
            .set_state(gst::State::Playing)
            .map_err(|e| RecorderError::PipelineError(e.to_string()))?;

        Ok(Self {
            pipeline,
            appsrc,
            camera_id,
            start_time: std::time::Instant::now(),
            frame_count: Arc::new(Mutex::new(0)),
            fps,
            width,
            height,
            output_path,
            is_recording: Arc::new(Mutex::new(true)),
        })
    }

    pub fn write_frame(&mut self, frame_data: &[u8]) -> Result<()> {
        if !*self.is_recording.lock().unwrap() {
            return Ok(());
        }

        let buffer_size = (self.width * self.height * 3) as usize;
        if frame_data.len() < buffer_size {
            return Err(RecorderError::WriteError(format!(
                "Frame zu klein: {} < {}",
                frame_data.len(),
                buffer_size
            )));
        }

        let mut buffer = gst::Buffer::with_size(buffer_size)
            .map_err(|e| RecorderError::WriteError(e.to_string()))?;

        {
            let buffer_ref = buffer.get_mut().unwrap();
            let mut map = buffer_ref
                .map_writable()
                .map_err(|e| RecorderError::WriteError(e.to_string()))?;
            map.copy_from_slice(&frame_data[..buffer_size]);
        }

        let mut count = self.frame_count.lock().unwrap();
        let pts = gst::ClockTime::from_seconds(*count) / (self.fps as u64);
        buffer.get_mut().unwrap().set_pts(pts);
        *count += 1;

        self.appsrc
            .push_buffer(buffer)
            .map_err(|e| RecorderError::WriteError(e.to_string()))?;

        Ok(())
    }

    pub fn finalize(self) -> Result<RecordingMetadata> {
        *self.is_recording.lock().unwrap() = false;

        let duration = self.start_time.elapsed().as_secs_f64();
        let frame_count = *self.frame_count.lock().unwrap();

        // Send EOS
        self.appsrc
            .end_of_stream()
            .map_err(|e| RecorderError::GStreamerError(e.to_string()))?;

        // Warte auf EOS
        let bus = self
            .pipeline
            .bus()
            .ok_or_else(|| RecorderError::PipelineError("Kein Bus".to_string()))?;

        for msg in bus.iter_timed(gst::ClockTime::from_seconds(5)) {
            use gst::MessageView;

            match msg.view() {
                MessageView::Eos(..) => break,
                MessageView::Error(err) => {
                    return Err(RecorderError::GStreamerError(format!(
                        "{:?}",
                        err.error()
                    )));
                }
                _ => (),
            }
        }

        // Stoppe Pipeline
        self.pipeline
            .set_state(gst::State::Null)
            .map_err(|e| RecorderError::PipelineError(e.to_string()))?;

        let metadata = RecordingMetadata {
            camera_id: self.camera_id,
            timestamp: Local::now().to_rfc3339(),
            duration_secs: duration,
            fps: self.fps,
            width: self.width,
            height: self.height,
            filename: self
                .output_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        };

        let metadata_path = self.output_path.with_extension("json");
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| RecorderError::MetadataError(e.to_string()))?;
        fs::write(&metadata_path, metadata_json)?;

        println!(
            "Aufnahme gespeichert: {} ({} frames, {:.2}s)",
            self.output_path.display(),
            frame_count,
            duration
        );

        Ok(metadata)
    }
}
