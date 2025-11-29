use crate::camera::CameraDevice;
use crate::recorder::VideoRecorder;
use crate::virtual_camera::VirtualCamera;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DualRecorderError {
    #[error("Kamera Fehler: {0}")]
    CameraError(String),
    #[error("Recorder Fehler: {0}")]
    RecorderError(String),
}

pub type Result<T> = std::result::Result<T, DualRecorderError>;

pub enum CameraSource {
    Real(u32, u32),           // (camera_0_id, camera_1_id)
    Virtual,                  // Two virtual test cameras
    Mixed(u32, bool),         // (real_camera_id, is_left) + virtual
}

pub struct DualCameraRecorder {
    running: Arc<AtomicBool>,
    left_frames: Arc<Mutex<Option<Vec<u8>>>>,
    right_frames: Arc<Mutex<Option<Vec<u8>>>>,
}

impl DualCameraRecorder {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            left_frames: Arc::new(Mutex::new(None)),
            right_frames: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_recording(
        &mut self,
        source: CameraSource,
        output_dir: &Path,
        fps: f64,
        duration_secs: u64,
    ) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            return Err(DualRecorderError::RecorderError(
                "Aufnahme läuft bereits".to_string(),
            ));
        }

        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let left_frames = self.left_frames.clone();
        let right_frames = self.right_frames.clone();
        let output_dir = output_dir.to_path_buf();

        thread::spawn(move || {
            if let Err(e) = Self::recording_thread(
                source,
                &output_dir,
                fps,
                duration_secs,
                running,
                left_frames,
                right_frames,
            ) {
                eprintln!("Aufnahme-Fehler: {}", e);
            }
        });

        Ok(())
    }

    fn recording_thread(
        source: CameraSource,
        output_dir: &Path,
        fps: f64,
        duration_secs: u64,
        running: Arc<AtomicBool>,
        left_frames: Arc<Mutex<Option<Vec<u8>>>>,
        right_frames: Arc<Mutex<Option<Vec<u8>>>>,
    ) -> Result<()> {
        match source {
            CameraSource::Real(cam0_id, cam1_id) => {
                Self::record_real_cameras(
                    cam0_id,
                    cam1_id,
                    output_dir,
                    fps,
                    duration_secs,
                    running,
                    left_frames,
                    right_frames,
                )
            }
            CameraSource::Virtual => {
                Self::record_virtual_cameras(
                    output_dir,
                    fps,
                    duration_secs,
                    running,
                    left_frames,
                    right_frames,
                )
            }
            CameraSource::Mixed(_, _) => {
                Err(DualRecorderError::RecorderError(
                    "Mixed mode noch nicht implementiert".to_string(),
                ))
            }
        }
    }

    fn record_real_cameras(
        cam0_id: u32,
        cam1_id: u32,
        output_dir: &Path,
        fps: f64,
        duration_secs: u64,
        running: Arc<AtomicBool>,
        left_frames: Arc<Mutex<Option<Vec<u8>>>>,
        right_frames: Arc<Mutex<Option<Vec<u8>>>>,
    ) -> Result<()> {
        println!("Starte Aufnahme von Kameras {} und {}", cam0_id, cam1_id);

        // Öffne beide Kameras
        let mut cam0 = CameraDevice::new(cam0_id)
            .map_err(|e| DualRecorderError::CameraError(format!("Kamera {}: {}", cam0_id, e)))?;
        let mut cam1 = CameraDevice::new(cam1_id)
            .map_err(|e| DualRecorderError::CameraError(format!("Kamera {}: {}", cam1_id, e)))?;

        cam0.start()
            .map_err(|e| DualRecorderError::CameraError(e.to_string()))?;
        cam1.start()
            .map_err(|e| DualRecorderError::CameraError(e.to_string()))?;

        // Erstelle Recorder
        let mut recorder0 = VideoRecorder::new(cam0_id, 640, 480, fps, output_dir)
            .map_err(|e| DualRecorderError::RecorderError(e.to_string()))?;
        let mut recorder1 = VideoRecorder::new(cam1_id, 640, 480, fps, output_dir)
            .map_err(|e| DualRecorderError::RecorderError(e.to_string()))?;

        let start = std::time::Instant::now();
        let frame_delay = Duration::from_millis((1000.0 / fps) as u64);

        while running.load(Ordering::SeqCst) && start.elapsed().as_secs() < duration_secs {
            // Lese Frames
            if let Ok(frame0) = cam0.get_frame() {
                *left_frames.lock().unwrap() = Some(frame0.clone());
                let _ = recorder0.write_frame(&frame0);
            }

            if let Ok(frame1) = cam1.get_frame() {
                *right_frames.lock().unwrap() = Some(frame1.clone());
                let _ = recorder1.write_frame(&frame1);
            }

            thread::sleep(frame_delay);
        }

        // Cleanup
        let _ = cam0.stop();
        let _ = cam1.stop();
        let _ = recorder0.finalize();
        let _ = recorder1.finalize();

        running.store(false, Ordering::SeqCst);
        println!("Aufnahme beendet");

        Ok(())
    }

    fn record_virtual_cameras(
        output_dir: &Path,
        fps: f64,
        duration_secs: u64,
        running: Arc<AtomicBool>,
        left_frames: Arc<Mutex<Option<Vec<u8>>>>,
        right_frames: Arc<Mutex<Option<Vec<u8>>>>,
    ) -> Result<()> {
        println!("Starte Aufnahme von virtuellen Kameras");

        let vcam0 = VirtualCamera::new(0, 640, 480, fps as u32);
        let vcam1 = VirtualCamera::new(1, 640, 480, fps as u32);

        let mut recorder0 = VideoRecorder::new(0, 640, 480, fps, output_dir)
            .map_err(|e| DualRecorderError::RecorderError(e.to_string()))?;
        let mut recorder1 = VideoRecorder::new(1, 640, 480, fps, output_dir)
            .map_err(|e| DualRecorderError::RecorderError(e.to_string()))?;

        let start = std::time::Instant::now();

        while running.load(Ordering::SeqCst) && start.elapsed().as_secs() < duration_secs {
            if let Ok(frame0) = vcam0.get_frame() {
                *left_frames.lock().unwrap() = Some(frame0.clone());
                let _ = recorder0.write_frame(&frame0);
            }

            if let Ok(frame1) = vcam1.get_frame() {
                *right_frames.lock().unwrap() = Some(frame1.clone());
                let _ = recorder1.write_frame(&frame1);
            }

            vcam0.wait_for_next_frame();
        }

        let _ = recorder0.finalize();
        let _ = recorder1.finalize();

        running.store(false, Ordering::SeqCst);
        println!("Aufnahme beendet");

        Ok(())
    }

    pub fn stop_recording(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_recording(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn get_left_frame(&self) -> Option<Vec<u8>> {
        self.left_frames.lock().unwrap().clone()
    }

    pub fn get_right_frame(&self) -> Option<Vec<u8>> {
        self.right_frames.lock().unwrap().clone()
    }
}
