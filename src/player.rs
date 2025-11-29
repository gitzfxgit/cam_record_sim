use gstreamer as gst;
use gstreamer::prelude::*;
use std::path::Path;
use std::thread;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlayerError {
    #[error("Video konnte nicht geöffnet werden: {0}")]
    OpenError(String),
    #[error("GStreamer Fehler: {0}")]
    GStreamerError(String),
    #[error("Pipeline Fehler: {0}")]
    PipelineError(String),
}

pub type Result<T> = std::result::Result<T, PlayerError>;

pub struct VideoPlayer {
    pipeline: gst::Pipeline,
    duration: Option<gst::ClockTime>,
}

impl VideoPlayer {
    pub fn new(video_path: &Path) -> Result<Self> {
        if !video_path.exists() {
            return Err(PlayerError::OpenError(format!(
                "Datei nicht gefunden: {}",
                video_path.display()
            )));
        }

        // Initialisiere GStreamer
        gst::init().map_err(|e| PlayerError::GStreamerError(e.to_string()))?;

        // Erstelle Pipeline: filesrc ! decodebin ! autovideosink
        let pipeline_str = format!(
            "filesrc location={} ! decodebin ! videoconvert ! autovideosink",
            video_path.to_str().unwrap()
        );

        let pipeline = gst::parse::launch(&pipeline_str)
            .map_err(|e| PlayerError::PipelineError(e.to_string()))?
            .downcast::<gst::Pipeline>()
            .map_err(|_| PlayerError::PipelineError("Kein Pipeline Element".to_string()))?;

        Ok(Self {
            pipeline,
            duration: None,
        })
    }

    pub fn play(&mut self) -> Result<()> {
        // Starte Pipeline
        self.pipeline
            .set_state(gst::State::Playing)
            .map_err(|e| PlayerError::PipelineError(e.to_string()))?;

        // Hole Duration
        let mut retries = 10;
        while self.duration.is_none() && retries > 0 {
            if let Some(duration) = self.pipeline.query_duration::<gst::ClockTime>() {
                self.duration = Some(duration);
                println!(
                    "Video-Dauer: {:.2} Sekunden",
                    duration.seconds() as f64
                );
            } else {
                thread::sleep(Duration::from_millis(100));
                retries -= 1;
            }
        }

        // Warte auf EOS oder Fehler
        let bus = self
            .pipeline
            .bus()
            .ok_or_else(|| PlayerError::PipelineError("Kein Bus".to_string()))?;

        for msg in bus.iter_timed(gst::ClockTime::NONE) {
            use gst::MessageView;

            match msg.view() {
                MessageView::Eos(..) => {
                    println!("Video Ende erreicht");
                    break;
                }
                MessageView::Error(err) => {
                    return Err(PlayerError::GStreamerError(format!(
                        "Fehler: {:?}",
                        err.error()
                    )));
                }
                MessageView::StateChanged(state) => {
                    if let Some(element) = state.src() {
                        if element == &self.pipeline {
                            println!("Pipeline State: {:?} -> {:?}", state.old(), state.current());
                        }
                    }
                }
                _ => (),
            }
        }

        self.pipeline
            .set_state(gst::State::Null)
            .map_err(|e| PlayerError::PipelineError(e.to_string()))?;

        Ok(())
    }

    pub fn get_fps(&self) -> f64 {
        30.0 // Default, könnte aus Stream-Info gelesen werden
    }

    pub fn get_frame_count(&self) -> i32 {
        if let Some(duration) = self.duration {
            (duration.seconds() as f64 * self.get_fps()) as i32
        } else {
            0
        }
    }

    pub fn reset(&mut self) -> Result<()> {
        self.pipeline
            .seek_simple(
                gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                gst::ClockTime::ZERO,
            )
            .map_err(|e| PlayerError::PipelineError(e.to_string()))?;
        Ok(())
    }
}

pub fn list_recordings(recordings_dir: &Path) -> Result<Vec<String>> {
    if !recordings_dir.exists() {
        return Ok(Vec::new());
    }

    let mut recordings = Vec::new();

    for entry in std::fs::read_dir(recordings_dir)
        .map_err(|e| PlayerError::OpenError(e.to_string()))?
    {
        let entry = entry.map_err(|e| PlayerError::OpenError(e.to_string()))?;
        let path = entry.path();

        if let Some(ext) = path.extension() {
            if ext == "mp4" || ext == "avi" || ext == "mkv" {
                recordings.push(path.file_name().unwrap().to_string_lossy().to_string());
            }
        }
    }

    recordings.sort();
    Ok(recordings)
}
