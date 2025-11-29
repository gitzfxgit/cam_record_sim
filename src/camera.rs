use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CameraError {
    #[error("Kamera konnte nicht geöffnet werden: {0}")]
    OpenError(String),
    #[error("Frame konnte nicht gelesen werden: {0}")]
    FrameError(String),
    #[error("Kamera nicht gefunden")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, CameraError>;

pub struct CameraDevice {
    camera: Arc<Mutex<Camera>>,
    index: u32,
}

impl CameraDevice {
    pub fn new(index: u32) -> Result<Self> {
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

        let camera = Camera::new(CameraIndex::Index(index), requested)
            .map_err(|e| CameraError::OpenError(e.to_string()))?;

        Ok(Self {
            camera: Arc::new(Mutex::new(camera)),
            index,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        self.camera
            .lock()
            .unwrap()
            .open_stream()
            .map_err(|e| CameraError::OpenError(e.to_string()))
    }

    pub fn get_frame(&self) -> Result<Vec<u8>> {
        let mut cam = self.camera.lock().unwrap();
        let frame = cam
            .frame()
            .map_err(|e| CameraError::FrameError(e.to_string()))?;

        Ok(frame.buffer().to_vec())
    }

    pub fn get_index(&self) -> u32 {
        self.index
    }

    pub fn stop(&mut self) -> Result<()> {
        self.camera
            .lock()
            .unwrap()
            .stop_stream()
            .map_err(|e| CameraError::OpenError(e.to_string()))
    }
}

pub fn list_cameras() -> Vec<u32> {
    (0..10)
        .filter(|&i| {
            Camera::new(
                CameraIndex::Index(i),
                RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
            )
            .is_ok()
        })
        .collect()
}
