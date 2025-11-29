mod camera;
mod player;
mod recorder;
mod virtual_camera;
mod playback_camera;
mod dual_recorder;
mod gui;

use camera::{CameraDevice, list_cameras};
use clap::{Parser, Subcommand};
use player::{VideoPlayer, list_recordings};
use recorder::VideoRecorder;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use virtual_camera::{VirtualCamera, create_virtual_cameras};

#[derive(Parser)]
#[command(name = "cam_record_sim")]
#[command(about = "Kamera-Aufnahme und Simulations-Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Listet alle verfügbaren echten Kameras auf")]
    ListCameras,

    #[command(about = "Nimmt von einer echten Kamera auf")]
    Record {
        #[arg(short, long, help = "Kamera-Index (0, 1, ...)")]
        camera: u32,

        #[arg(short, long, default_value = "recordings", help = "Ausgabe-Verzeichnis")]
        output: PathBuf,

        #[arg(short, long, default_value = "30.0", help = "Frames pro Sekunde")]
        fps: f64,

        #[arg(short, long, default_value = "60", help = "Aufnahmedauer in Sekunden")]
        duration: u64,
    },

    #[command(about = "Startet virtuelle Kamera-Simulation und nimmt auf")]
    SimRecord {
        #[arg(short, long, default_value = "0", help = "Virtuelle Kamera ID (0 oder 1)")]
        camera: u32,

        #[arg(short, long, default_value = "recordings", help = "Ausgabe-Verzeichnis")]
        output: PathBuf,

        #[arg(short, long, default_value = "30.0", help = "Frames pro Sekunde")]
        fps: f64,

        #[arg(short, long, default_value = "10", help = "Aufnahmedauer in Sekunden")]
        duration: u64,
    },

    #[command(about = "Listet alle Aufnahmen auf")]
    ListRecordings {
        #[arg(short, long, default_value = "recordings", help = "Aufnahme-Verzeichnis")]
        dir: PathBuf,
    },

    #[command(about = "Spielt eine Aufnahme ab")]
    Play {
        #[arg(help = "Dateiname der Aufnahme")]
        file: String,

        #[arg(short, long, default_value = "recordings", help = "Aufnahme-Verzeichnis")]
        dir: PathBuf,
    },

    #[command(about = "Testet zwei virtuelle Kameras")]
    TestVirtual {
        #[arg(short, long, default_value = "5", help = "Testdauer in Sekunden")]
        duration: u64,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Wenn kein Kommando angegeben wurde, starte GUI
    if cli.command.is_none() {
        gui::run_gui();
        return Ok(());
    }

    match cli.command.unwrap() {
        Commands::ListCameras => {
            println!("Suche nach verfügbaren Kameras...");
            let cameras = list_cameras();

            if cameras.is_empty() {
                println!("Keine Kameras gefunden!");
            } else {
                println!("Gefundene Kameras:");
                for cam_id in cameras {
                    println!("  - Kamera {}", cam_id);
                }
            }
        }

        Commands::Record {
            camera,
            output,
            fps,
            duration,
        } => {
            println!("Öffne Kamera {}...", camera);
            let mut cam = CameraDevice::new(camera)?;
            cam.start()?;

            println!("Starte Aufnahme für {} Sekunden...", duration);
            let mut recorder = VideoRecorder::new(camera, 640, 480, fps, &output)?;

            let running = Arc::new(AtomicBool::new(true));
            let r = running.clone();

            ctrlc::set_handler(move || {
                println!("\nStoppe Aufnahme...");
                r.store(false, Ordering::SeqCst);
            })
            .expect("Fehler beim Setzen des Ctrl-C Handlers");

            let start = std::time::Instant::now();
            let mut frame_count = 0;

            while running.load(Ordering::SeqCst) && start.elapsed().as_secs() < duration {
                match cam.get_frame() {
                    Ok(frame) => {
                        recorder.write_frame(&frame)?;
                        frame_count += 1;

                        if frame_count % 30 == 0 {
                            println!("Aufgenommen: {} frames", frame_count);
                        }
                    }
                    Err(e) => {
                        eprintln!("Fehler beim Lesen des Frames: {}", e);
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(
                    (1000.0 / fps) as u64,
                ));
            }

            cam.stop()?;
            let metadata = recorder.finalize()?;
            println!("Aufnahme abgeschlossen!");
            println!("  Datei: {}", metadata.filename);
            println!("  Dauer: {:.2}s", metadata.duration_secs);
            println!("  Frames: {}", frame_count);
        }

        Commands::SimRecord {
            camera,
            output,
            fps,
            duration,
        } => {
            if camera > 1 {
                println!("Virtuelle Kamera ID muss 0 oder 1 sein!");
                return Ok(());
            }

            println!("Starte virtuelle Kamera {}...", camera);
            let vcam = VirtualCamera::new(camera, 640, 480, fps as u32);

            println!("Starte Aufnahme für {} Sekunden...", duration);
            let mut recorder = VideoRecorder::new(camera, 640, 480, fps, &output)?;

            let running = Arc::new(AtomicBool::new(true));
            let r = running.clone();

            ctrlc::set_handler(move || {
                println!("\nStoppe Aufnahme...");
                r.store(false, Ordering::SeqCst);
            })
            .expect("Fehler beim Setzen des Ctrl-C Handlers");

            let start = std::time::Instant::now();
            let mut frame_count = 0;

            while running.load(Ordering::SeqCst) && start.elapsed().as_secs() < duration {
                match vcam.get_frame() {
                    Ok(frame) => {
                        recorder.write_frame(&frame)?;
                        frame_count += 1;

                        if frame_count % 30 == 0 {
                            println!("Aufgenommen: {} frames", frame_count);
                        }
                    }
                    Err(e) => {
                        eprintln!("Fehler beim Generieren des Frames: {}", e);
                    }
                }

                vcam.wait_for_next_frame();
            }

            let metadata = recorder.finalize()?;
            println!("Aufnahme abgeschlossen!");
            println!("  Datei: {}", metadata.filename);
            println!("  Dauer: {:.2}s", metadata.duration_secs);
            println!("  Frames: {}", frame_count);
        }

        Commands::ListRecordings { dir } => {
            println!("Aufnahmen in {:?}:", dir);
            let recordings = list_recordings(&dir)?;

            if recordings.is_empty() {
                println!("  Keine Aufnahmen gefunden!");
            } else {
                for (i, rec) in recordings.iter().enumerate() {
                    println!("  {}. {}", i + 1, rec);
                }
            }
        }

        Commands::Play { file, dir } => {
            let video_path = dir.join(&file);
            println!("Lade Video: {:?}", video_path);

            let mut player = VideoPlayer::new(&video_path)?;
            player.play()?;
        }

        Commands::TestVirtual { duration } => {
            println!("Starte Test mit zwei virtuellen Kameras...");
            let vcams = create_virtual_cameras();

            println!("Virtuelle Kameras erstellt:");
            for vcam in &vcams {
                let (w, h) = vcam.get_resolution();
                println!(
                    "  - Kamera {} ({}x{} @ {} FPS)",
                    vcam.get_id(),
                    w,
                    h,
                    vcam.get_fps()
                );
            }

            println!("\nGeneriere Frames für {} Sekunden...", duration);
            let start = std::time::Instant::now();
            let mut frame_counts = vec![0; vcams.len()];

            while start.elapsed().as_secs() < duration {
                for (i, vcam) in vcams.iter().enumerate() {
                    match vcam.get_frame() {
                        Ok(_frame) => {
                            frame_counts[i] += 1;
                        }
                        Err(e) => {
                            eprintln!("Fehler bei Kamera {}: {}", i, e);
                        }
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(33)); // ~30 FPS
            }

            println!("\nTest abgeschlossen!");
            for (i, count) in frame_counts.iter().enumerate() {
                println!("  Kamera {}: {} Frames generiert", i, count);
            }
        }
    }

    Ok(())
}
