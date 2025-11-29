use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VirtualCameraError {
    #[error("Virtuelle Kamera konnte nicht erstellt werden")]
    CreationError,
    #[error("Frame konnte nicht generiert werden")]
    FrameError,
}

pub type Result<T> = std::result::Result<T, VirtualCameraError>;

pub struct VirtualCamera {
    id: u32,
    width: u32,
    height: u32,
    fps: u32,
    frame_count: Arc<Mutex<u64>>,
    start_time: Instant,
}

impl VirtualCamera {
    pub fn new(id: u32, width: u32, height: u32, fps: u32) -> Self {
        Self {
            id,
            width,
            height,
            fps,
            frame_count: Arc::new(Mutex::new(0)),
            start_time: Instant::now(),
        }
    }

    pub fn get_frame(&self) -> Result<Vec<u8>> {
        let mut count = self.frame_count.lock().unwrap();
        *count += 1;

        // Generiere ein Test-Pattern (bewegende Balken)
        let frame_size = (self.width * self.height * 3) as usize;
        let mut frame = vec![0u8; frame_size];

        let elapsed = self.start_time.elapsed().as_secs_f32();
        let offset = ((elapsed * 50.0) as u32) % self.width;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = ((y * self.width + x) * 3) as usize;

                // Farbiger bewegender Balken basierend auf Kamera-ID
                let pattern = (x + offset) % (self.width / 4);

                if pattern < 10 {
                    match self.id % 3 {
                        0 => {
                            frame[idx] = 255;     // Rot
                            frame[idx + 1] = 0;
                            frame[idx + 2] = 0;
                        }
                        1 => {
                            frame[idx] = 0;
                            frame[idx + 1] = 255; // Grün
                            frame[idx + 2] = 0;
                        }
                        _ => {
                            frame[idx] = 0;
                            frame[idx + 1] = 0;
                            frame[idx + 2] = 255; // Blau
                        }
                    }
                } else {
                    frame[idx] = 50;
                    frame[idx + 1] = 50;
                    frame[idx + 2] = 50;
                }
            }
        }

        Ok(frame)
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_fps(&self) -> u32 {
        self.fps
    }

    pub fn wait_for_next_frame(&self) {
        let frame_duration = Duration::from_millis(1000 / self.fps as u64);
        std::thread::sleep(frame_duration);
    }
}

pub fn create_virtual_cameras() -> Vec<VirtualCamera> {
    vec![
        VirtualCamera::new(0, 640, 480, 30),
        VirtualCamera::new(1, 640, 480, 30),
    ]
}
