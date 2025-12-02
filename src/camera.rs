use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType, Resolution};
use nokhwa::Camera;
use std::collections::HashSet;
use std::fs;
use std::process::Command;
use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::gst_camera::{GstCamera, is_bayer_camera, detect_bayer_format};

#[derive(Error, Debug)]
pub enum CameraError {
    #[error("Camera could not be opened: {0}")]
    OpenError(String),
    #[error("Frame could not be read: {0}")]
    FrameError(String),
    #[error("Camera not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, CameraError>;

enum CameraBackend {
    Nokhwa(Arc<Mutex<Camera>>),
    GStreamer(Arc<Mutex<GstCamera>>),
}

pub struct CameraDevice {
    backend: CameraBackend,
    index: u32,
}

impl CameraDevice {
    pub fn new(index: u32) -> Result<Self> {
        Self::new_with_resolution(index, 640, 480)
    }

    pub fn new_with_resolution(index: u32, width: u32, height: u32) -> Result<Self> {
        // First, check if this is a Bayer format camera (like The Imaging Source DFK 37BUX265)
        if is_bayer_camera(index) {
            eprintln!("Using GStreamer backend for Bayer camera {} ({}x{})", index, width, height);
            match GstCamera::new(index, width, height, 30) {
                Ok(gst_cam) => {
                    return Ok(Self {
                        backend: CameraBackend::GStreamer(Arc::new(Mutex::new(gst_cam))),
                        index,
                    });
                }
                Err(e) => {
                    eprintln!("Failed to open camera {} with GStreamer: {}", index, e);
                    eprintln!("Falling back to nokhwa...");
                }
            }
        }

        // Try different format types to support both regular USB cameras and industrial cameras
        let format_types = vec![
            RequestedFormatType::None,           // Let the camera choose
            RequestedFormatType::AbsoluteHighestFrameRate, // High FPS
            RequestedFormatType::AbsoluteHighestResolution, // High resolution
        ];

        let mut last_error = String::new();

        for format_type in format_types {
            let requested = RequestedFormat::new::<RgbFormat>(format_type);

            match Camera::new(CameraIndex::Index(index), requested) {
                Ok(mut camera) => {
                    // Try to set resolution, but don't fail if it doesn't work
                    if let Err(e) = camera.set_resolution(Resolution::new(width, height)) {
                        eprintln!("Warning: Could not set resolution {}x{} for camera {}: {}", width, height, index, e);
                    }

                    eprintln!("Using nokhwa backend for camera {} ({}x{})", index, width, height);
                    return Ok(Self {
                        backend: CameraBackend::Nokhwa(Arc::new(Mutex::new(camera))),
                        index,
                    });
                }
                Err(e) => {
                    last_error = format!("{}", e);
                    eprintln!("Attempt failed with format type {:?}: {}", format_type, e);
                }
            }
        }

        Err(CameraError::OpenError(format!(
            "Camera {}: All format attempts failed. Last error: {}",
            index,
            last_error
        )))
    }

    pub fn start(&mut self) -> Result<()> {
        match &self.backend {
            CameraBackend::Nokhwa(camera) => {
                camera
                    .lock()
                    .unwrap()
                    .open_stream()
                    .map_err(|e| CameraError::OpenError(e.to_string()))
            }
            CameraBackend::GStreamer(gst_cam) => {
                gst_cam
                    .lock()
                    .unwrap()
                    .start()
                    .map_err(|e| CameraError::OpenError(e.to_string()))
            }
        }
    }

    pub fn get_frame(&self) -> Result<Vec<u8>> {
        match &self.backend {
            CameraBackend::Nokhwa(camera) => {
                let mut cam = camera.lock().unwrap();
                let frame = cam
                    .frame()
                    .map_err(|e| CameraError::FrameError(e.to_string()))?;

                let decoded = frame.decode_image::<nokhwa::pixel_format::RgbFormat>()
                    .map_err(|e| CameraError::FrameError(e.to_string()))?;

                Ok(decoded.into_flat_samples().samples)
            }
            CameraBackend::GStreamer(gst_cam) => {
                gst_cam
                    .lock()
                    .unwrap()
                    .get_frame()
                    .map_err(|e| CameraError::FrameError(e.to_string()))
            }
        }
    }

    pub fn get_index(&self) -> u32 {
        self.index
    }

    pub fn stop(&mut self) -> Result<()> {
        match &self.backend {
            CameraBackend::Nokhwa(camera) => {
                camera
                    .lock()
                    .unwrap()
                    .stop_stream()
                    .map_err(|e| CameraError::OpenError(e.to_string()))
            }
            CameraBackend::GStreamer(gst_cam) => {
                gst_cam
                    .lock()
                    .unwrap()
                    .stop()
                    .map_err(|e| CameraError::OpenError(e.to_string()))
            }
        }
    }
}

pub struct CameraInfo {
    pub index: u32,
    pub name: String,
}

fn get_v4l2_device_name(device_path: &str) -> Option<String> {
    let output = Command::new("v4l2-ctl")
        .arg("--device")
        .arg(device_path)
        .arg("--info")
        .output()
        .ok()?;

    if output.status.success() {
        let info = String::from_utf8_lossy(&output.stdout);
        for line in info.lines() {
            if line.contains("Card type") {
                if let Some(name) = line.split(':').nth(1) {
                    return Some(name.trim().to_string());
                }
            }
        }
    }
    None
}

fn extract_device_index(device_name: &str) -> Option<u32> {
    // Handle both "video0" and "/dev/video0" formats
    device_name
        .trim_start_matches("/dev/video")
        .trim_start_matches("video")
        .parse::<u32>()
        .ok()
}

pub fn list_cameras() -> Vec<CameraInfo> {
    let mut cameras = Vec::new();
    let mut found_devices = HashSet::new();

    // Try to enumerate V4L2 devices directly (for industrial cameras like DFK 37BUX265)
    if let Ok(entries) = fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();
                if filename_str.starts_with("video") {
                    let device_path = path.to_string_lossy().to_string();

                    if let Some(index) = extract_device_index(&filename_str) {
                        if found_devices.contains(&index) {
                            continue;
                        }

                        // Try to get device name via v4l2-ctl
                        let name = get_v4l2_device_name(&device_path)
                            .unwrap_or_else(|| format!("Video Device {}", index));

                        cameras.push(CameraInfo {
                            index,
                            name: format!("{} (/dev/video{})", name, index),
                        });
                        found_devices.insert(index);
                    }
                }
            }
        }
    }

    // Also try nokhwa for standard USB cameras
    for i in 0..20 {
        if found_devices.contains(&i) {
            continue;
        }

        if let Ok(camera) = Camera::new(
            CameraIndex::Index(i),
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
        ) {
            let name = camera.info().human_name();
            let display_name = if name.is_empty() {
                format!("Camera {}", i)
            } else {
                name
            };

            cameras.push(CameraInfo {
                index: i,
                name: format!("{} (ID: {})", display_name, i),
            });
            found_devices.insert(i);
        }
    }

    // Sort by index
    cameras.sort_by_key(|c| c.index);
    cameras
}
