use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlaybackCameraError {
    #[error("Video konnte nicht geöffnet werden: {0}")]
    OpenError(String),
    #[error("Frame konnte nicht gelesen werden: {0}")]
    ReadError(String),
    #[error("GStreamer Fehler: {0}")]
    GStreamerError(String),
}

pub type Result<T> = std::result::Result<T, PlaybackCameraError>;

/// Virtuelle Kamera die eine Video-Datei als Input verwendet
pub struct PlaybackCamera {
    pipeline: gst::Pipeline,
    appsink: gst_app::AppSink,
    camera_id: u32,
    video_path: PathBuf,
    loop_playback: bool,
    frame_count: Arc<Mutex<i32>>,
    current_frame: Arc<Mutex<i32>>,
}

impl PlaybackCamera {
    pub fn new(camera_id: u32, video_path: &Path, loop_playback: bool) -> Result<Self> {
        if !video_path.exists() {
            return Err(PlaybackCameraError::OpenError(format!(
                "Datei nicht gefunden: {}",
                video_path.display()
            )));
        }

        // Initialisiere GStreamer
        gst::init().map_err(|e| PlaybackCameraError::GStreamerError(e.to_string()))?;

        // Erstelle Pipeline: filesrc ! decodebin ! videoconvert ! appsink
        let pipeline_str = format!(
            "filesrc location={} ! decodebin ! videoconvert ! video/x-raw,format=RGB ! appsink name=sink",
            video_path.to_str().unwrap()
        );

        let pipeline = gst::parse::launch(&pipeline_str)
            .map_err(|e| PlaybackCameraError::GStreamerError(e.to_string()))?
            .downcast::<gst::Pipeline>()
            .map_err(|_| PlaybackCameraError::GStreamerError("Kein Pipeline Element".to_string()))?;

        let appsink = pipeline
            .by_name("sink")
            .ok_or_else(|| PlaybackCameraError::GStreamerError("appsink nicht gefunden".to_string()))?
            .downcast::<gst_app::AppSink>()
            .map_err(|_| PlaybackCameraError::GStreamerError("Kein AppSink Element".to_string()))?;

        // Starte Pipeline
        pipeline
            .set_state(gst::State::Playing)
            .map_err(|e| PlaybackCameraError::GStreamerError(e.to_string()))?;

        Ok(Self {
            pipeline,
            appsink,
            camera_id,
            video_path: video_path.to_path_buf(),
            loop_playback,
            frame_count: Arc::new(Mutex::new(0)),
            current_frame: Arc::new(Mutex::new(0)),
        })
    }

    /// Liest den nächsten Frame und gibt ihn als RGB-Buffer zurück
    pub fn get_frame(&mut self) -> Result<Vec<u8>> {
        match self.appsink.try_pull_sample(gst::ClockTime::from_seconds(1)) {
            Some(sample) => {
                let buffer = sample.buffer().ok_or_else(|| {
                    PlaybackCameraError::ReadError("Kein Buffer im Sample".to_string())
                })?;

                let map = buffer.map_readable().map_err(|e| {
                    PlaybackCameraError::ReadError(format!("Buffer mapping fehlgeschlagen: {}", e))
                })?;

                let data = map.as_slice().to_vec();
                *self.current_frame.lock().unwrap() += 1;

                Ok(data)
            }
            None => {
                if self.loop_playback {
                    // Zurück zum Anfang
                    self.reset()?;
                    self.get_frame()
                } else {
                    Err(PlaybackCameraError::ReadError(
                        "Ende des Videos erreicht".to_string(),
                    ))
                }
            }
        }
    }

    pub fn reset(&mut self) -> Result<()> {
        self.pipeline
            .seek_simple(
                gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                gst::ClockTime::ZERO,
            )
            .map_err(|e| PlaybackCameraError::GStreamerError(e.to_string()))?;

        *self.current_frame.lock().unwrap() = 0;
        Ok(())
    }

    pub fn get_camera_id(&self) -> u32 {
        self.camera_id
    }

    pub fn get_fps(&self) -> f64 {
        30.0 // Default
    }

    pub fn get_frame_count(&self) -> i32 {
        *self.frame_count.lock().unwrap()
    }

    pub fn get_current_frame(&self) -> i32 {
        *self.current_frame.lock().unwrap()
    }

    pub fn get_progress(&self) -> f64 {
        let frame_count = self.get_frame_count();
        if frame_count > 0 {
            self.get_current_frame() as f64 / frame_count as f64
        } else {
            0.0
        }
    }

    pub fn is_finished(&self) -> bool {
        self.get_current_frame() >= self.get_frame_count()
    }

    pub fn get_video_path(&self) -> &Path {
        &self.video_path
    }
}

/// Verwaltet zwei PlaybackCameras (links und rechts) für Stereo-Simulation
pub struct StereoPlaybackSystem {
    left_camera: Option<PlaybackCamera>,
    right_camera: Option<PlaybackCamera>,
}

impl StereoPlaybackSystem {
    pub fn new() -> Self {
        Self {
            left_camera: None,
            right_camera: None,
        }
    }

    pub fn load_from_directory(recording_dir: &Path) -> Result<Self> {
        let mut system = Self::new();

        // Suche nach Aufnahmen im Verzeichnis
        let recordings = find_recordings_in_dir(recording_dir)?;

        if recordings.len() >= 2 {
            // Lade die ersten beiden Videos als linke und rechte Kamera
            system.left_camera = Some(PlaybackCamera::new(0, &recordings[0], true)?);
            system.right_camera = Some(PlaybackCamera::new(1, &recordings[1], true)?);

            println!("Linke Kamera: {}", recordings[0].display());
            println!("Rechte Kamera: {}", recordings[1].display());
        } else if recordings.len() == 1 {
            // Nur ein Video gefunden, nutze es für beide Kameras
            system.left_camera = Some(PlaybackCamera::new(0, &recordings[0], true)?);
            system.right_camera = Some(PlaybackCamera::new(1, &recordings[0], true)?);

            println!("Nur ein Video gefunden, wird für beide Kameras verwendet");
        } else {
            return Err(PlaybackCameraError::OpenError(
                "Keine Video-Dateien im Verzeichnis gefunden".to_string(),
            ));
        }

        Ok(system)
    }

    pub fn set_left_camera(&mut self, video_path: &Path) -> Result<()> {
        self.left_camera = Some(PlaybackCamera::new(0, video_path, true)?);
        Ok(())
    }

    pub fn set_right_camera(&mut self, video_path: &Path) -> Result<()> {
        self.right_camera = Some(PlaybackCamera::new(1, video_path, true)?);
        Ok(())
    }

    pub fn get_left_frame(&mut self) -> Result<Vec<u8>> {
        self.left_camera
            .as_mut()
            .ok_or_else(|| PlaybackCameraError::OpenError("Linke Kamera nicht geladen".to_string()))?
            .get_frame()
    }

    pub fn get_right_frame(&mut self) -> Result<Vec<u8>> {
        self.right_camera
            .as_mut()
            .ok_or_else(|| PlaybackCameraError::OpenError("Rechte Kamera nicht geladen".to_string()))?
            .get_frame()
    }

    pub fn get_both_frames(&mut self) -> Result<(Vec<u8>, Vec<u8>)> {
        let left = self.get_left_frame()?;
        let right = self.get_right_frame()?;
        Ok((left, right))
    }

    pub fn reset(&mut self) -> Result<()> {
        if let Some(cam) = &mut self.left_camera {
            cam.reset()?;
        }
        if let Some(cam) = &mut self.right_camera {
            cam.reset()?;
        }
        Ok(())
    }

    pub fn get_status(&self) -> String {
        let left_status = if let Some(cam) = &self.left_camera {
            format!(
                "Links: {} ({}/{})",
                cam.get_video_path().file_name().unwrap().to_string_lossy(),
                cam.get_current_frame(),
                cam.get_frame_count()
            )
        } else {
            "Links: Nicht geladen".to_string()
        };

        let right_status = if let Some(cam) = &self.right_camera {
            format!(
                "Rechts: {} ({}/{})",
                cam.get_video_path().file_name().unwrap().to_string_lossy(),
                cam.get_current_frame(),
                cam.get_frame_count()
            )
        } else {
            "Rechts: Nicht geladen".to_string()
        };

        format!("{}\n{}", left_status, right_status)
    }
}

fn find_recordings_in_dir(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Err(PlaybackCameraError::OpenError(format!(
            "Verzeichnis nicht gefunden: {}",
            dir.display()
        )));
    }

    let mut recordings = Vec::new();

    for entry in std::fs::read_dir(dir)
        .map_err(|e| PlaybackCameraError::OpenError(e.to_string()))?
    {
        let entry = entry.map_err(|e| PlaybackCameraError::OpenError(e.to_string()))?;
        let path = entry.path();

        if let Some(ext) = path.extension() {
            if ext == "mp4" || ext == "avi" || ext == "mkv" {
                recordings.push(path);
            }
        }
    }

    recordings.sort();
    Ok(recordings)
}
