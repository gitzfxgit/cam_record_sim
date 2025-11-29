use gtk4::prelude::*;
use gtk4::{
    gdk_pixbuf::{Colorspace, Pixbuf},
    glib, Application, ApplicationWindow, Box, Button, ComboBoxText, Entry, Image, Label,
    Notebook, Orientation, Separator, SpinButton,
};
use glib::Bytes;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::dual_recorder::{CameraSource, DualCameraRecorder};
use crate::player::list_recordings;
use crate::playback_camera::StereoPlaybackSystem;

const APP_ID: &str = "com.github.fasttube.CamRecordSim";

pub fn run_gui() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Camera Record Simulator")
        .default_width(1200)
        .default_height(800)
        .build();

    let main_box = Box::new(Orientation::Vertical, 10);
    main_box.set_margin_start(10);
    main_box.set_margin_end(10);
    main_box.set_margin_top(10);
    main_box.set_margin_bottom(10);

    // Header
    let header = Label::new(Some("<big><b>Dual-Kamera Aufnahme und Simulation</b></big>"));
    header.set_use_markup(true);
    main_box.append(&header);

    main_box.append(&Separator::new(Orientation::Horizontal));

    // Notebook für Tabs
    let notebook = Notebook::new();

    // Shared Recorder State
    let recorder = Rc::new(RefCell::new(DualCameraRecorder::new()));

    // Tab 1: Dual-Kamera Aufnahme mit Live-Feed
    let record_tab = create_dual_record_tab(recorder.clone());
    notebook.append_page(&record_tab, Some(&Label::new(Some("Aufnahme"))));

    // Tab 2: Simulation
    let simulation_tab = create_simulation_tab();
    notebook.append_page(&simulation_tab, Some(&Label::new(Some("Simulation"))));

    // Tab 3: Wiedergabe
    let playback_tab = create_playback_tab();
    notebook.append_page(&playback_tab, Some(&Label::new(Some("Wiedergabe"))));

    main_box.append(&notebook);

    window.set_child(Some(&main_box));
    window.present();
}

fn create_dual_record_tab(recorder: Rc<RefCell<DualCameraRecorder>>) -> Box {
    let tab_box = Box::new(Orientation::Vertical, 10);
    tab_box.set_margin_start(10);
    tab_box.set_margin_end(10);
    tab_box.set_margin_top(10);
    tab_box.set_margin_bottom(10);

    // Settings Box
    let settings_box = Box::new(Orientation::Horizontal, 10);

    // Left column - Camera selection
    let left_col = Box::new(Orientation::Vertical, 5);

    let cam_label = Label::new(Some("<b>Kamera-Typ:</b>"));
    cam_label.set_use_markup(true);
    cam_label.set_xalign(0.0);
    left_col.append(&cam_label);

    let camera_type = ComboBoxText::new();
    camera_type.append(Some("virtual"), "Virtuelle Test-Kameras");
    camera_type.append(Some("real"), "Zwei echte Kameras");
    camera_type.set_active(Some(0));
    left_col.append(&camera_type);

    // Camera IDs (only for real cameras)
    let cam_ids_box = Box::new(Orientation::Horizontal, 5);
    let cam0_label = Label::new(Some("Kamera 0 ID:"));
    let cam0_spin = SpinButton::with_range(0.0, 10.0, 1.0);
    cam0_spin.set_value(0.0);

    let cam1_label = Label::new(Some("Kamera 1 ID:"));
    let cam1_spin = SpinButton::with_range(0.0, 10.0, 1.0);
    cam1_spin.set_value(1.0);

    cam_ids_box.append(&cam0_label);
    cam_ids_box.append(&cam0_spin);
    cam_ids_box.append(&cam1_label);
    cam_ids_box.append(&cam1_spin);
    cam_ids_box.set_sensitive(false);

    // Show/hide camera IDs based on type
    let cam_ids_box_clone = cam_ids_box.clone();
    camera_type.connect_changed(move |combo| {
        if let Some(id) = combo.active_id() {
            cam_ids_box_clone.set_sensitive(id.as_str() == "real");
        }
    });

    left_col.append(&cam_ids_box);

    // FPS and Duration
    let fps_box = Box::new(Orientation::Horizontal, 5);
    let fps_label = Label::new(Some("FPS:"));
    let fps_spin = SpinButton::with_range(1.0, 60.0, 1.0);
    fps_spin.set_value(30.0);
    fps_box.append(&fps_label);
    fps_box.append(&fps_spin);
    left_col.append(&fps_box);

    let duration_box = Box::new(Orientation::Horizontal, 5);
    let duration_label = Label::new(Some("Dauer (Sek):"));
    let duration_spin = SpinButton::with_range(1.0, 300.0, 1.0);
    duration_spin.set_value(10.0);
    duration_box.append(&duration_label);
    duration_box.append(&duration_spin);
    left_col.append(&duration_box);

    // Output directory
    let output_box = Box::new(Orientation::Horizontal, 5);
    let output_label = Label::new(Some("Ausgabe:"));
    let output_entry = Entry::new();
    output_entry.set_text("recordings");
    output_entry.set_hexpand(true);
    output_box.append(&output_label);
    output_box.append(&output_entry);
    left_col.append(&output_box);

    settings_box.append(&left_col);

    tab_box.append(&settings_box);

    // Control Buttons
    let button_box = Box::new(Orientation::Horizontal, 5);

    let start_btn = Button::with_label("Aufnahme starten");
    start_btn.add_css_class("suggested-action");

    let stop_btn = Button::with_label("Aufnahme stoppen");
    stop_btn.add_css_class("destructive-action");
    stop_btn.set_sensitive(false);

    let status_label = Label::new(Some("Bereit"));
    status_label.set_margin_start(20);

    button_box.append(&start_btn);
    button_box.append(&stop_btn);
    button_box.append(&status_label);

    tab_box.append(&button_box);

    tab_box.append(&Separator::new(Orientation::Horizontal));

    // Live Preview
    let preview_label = Label::new(Some("<b>Live-Vorschau:</b>"));
    preview_label.set_use_markup(true);
    preview_label.set_xalign(0.0);
    tab_box.append(&preview_label);

    let preview_box = Box::new(Orientation::Horizontal, 10);
    preview_box.set_homogeneous(true);

    // Left camera preview
    let left_preview_box = Box::new(Orientation::Vertical, 5);
    let left_label = Label::new(Some("Linke Kamera (0)"));
    let left_image = Image::new();
    left_image.set_pixel_size(320);
    left_preview_box.append(&left_label);
    left_preview_box.append(&left_image);

    // Right camera preview
    let right_preview_box = Box::new(Orientation::Vertical, 5);
    let right_label = Label::new(Some("Rechte Kamera (1)"));
    let right_image = Image::new();
    right_image.set_pixel_size(320);
    right_preview_box.append(&right_label);
    right_preview_box.append(&right_image);

    preview_box.append(&left_preview_box);
    preview_box.append(&right_preview_box);

    tab_box.append(&preview_box);

    // Start Button Handler
    let recorder_clone = recorder.clone();
    let camera_type_clone = camera_type.clone();
    let cam0_spin_clone = cam0_spin.clone();
    let cam1_spin_clone = cam1_spin.clone();
    let fps_spin_clone = fps_spin.clone();
    let duration_spin_clone = duration_spin.clone();
    let output_entry_clone = output_entry.clone();
    let stop_btn_clone = stop_btn.clone();
    let status_label_clone = status_label.clone();
    let left_image_clone = left_image.clone();
    let right_image_clone = right_image.clone();

    start_btn.connect_clicked(move |btn| {
        let source = if camera_type_clone.active_id().unwrap().as_str() == "virtual" {
            CameraSource::Virtual
        } else {
            CameraSource::Real(
                cam0_spin_clone.value() as u32,
                cam1_spin_clone.value() as u32,
            )
        };

        let output_dir = PathBuf::from(output_entry_clone.text().as_str());
        let fps = fps_spin_clone.value();
        let duration = duration_spin_clone.value() as u64;

        match recorder_clone
            .borrow_mut()
            .start_recording(source, &output_dir, fps, duration)
        {
            Ok(_) => {
                btn.set_sensitive(false);
                stop_btn_clone.set_sensitive(true);
                status_label_clone.set_label("Aufnahme läuft...");

                // Start live preview update
                let recorder_preview = recorder_clone.clone();
                let left_img = left_image_clone.clone();
                let right_img = right_image_clone.clone();

                glib::timeout_add_local(std::time::Duration::from_millis(33), move || {
                    let rec = recorder_preview.borrow();

                    if !rec.is_recording() {
                        return glib::ControlFlow::Break;
                    }

                    // Update left preview
                    if let Some(frame) = rec.get_left_frame() {
                        if let Some(pixbuf) = frame_to_pixbuf(&frame, 640, 480) {
                            left_img.set_from_pixbuf(Some(&pixbuf));
                        }
                    }

                    // Update right preview
                    if let Some(frame) = rec.get_right_frame() {
                        if let Some(pixbuf) = frame_to_pixbuf(&frame, 640, 480) {
                            right_img.set_from_pixbuf(Some(&pixbuf));
                        }
                    }

                    glib::ControlFlow::Continue
                });
            }
            Err(e) => {
                status_label_clone.set_label(&format!("Fehler: {}", e));
            }
        }
    });

    // Stop Button Handler
    let recorder_clone2 = recorder.clone();
    let start_btn_clone = start_btn.clone();
    let status_label_clone2 = status_label.clone();

    stop_btn.connect_clicked(move |btn| {
        recorder_clone2.borrow_mut().stop_recording();
        btn.set_sensitive(false);
        start_btn_clone.set_sensitive(true);
        status_label_clone2.set_label("Aufnahme gestoppt");
    });

    tab_box
}

fn frame_to_pixbuf(frame: &[u8], width: i32, height: i32) -> Option<Pixbuf> {
    let expected_size = (width * height * 3) as usize;
    if frame.len() < expected_size {
        return None;
    }

    Some(Pixbuf::from_bytes(
        &Bytes::from(&frame[..expected_size]),
        Colorspace::Rgb,
        false,
        8,
        width,
        height,
        width * 3,
    ))
}

fn create_simulation_tab() -> Box {
    let tab_box = Box::new(Orientation::Vertical, 10);
    tab_box.set_margin_start(10);
    tab_box.set_margin_end(10);
    tab_box.set_margin_top(10);
    tab_box.set_margin_bottom(10);

    let desc = Label::new(Some(
        "<b>Simulation:</b> Wähle einen Ordner mit Aufzeichnungen.\n\
        Die erste Aufnahme wird als linke virtuelle Kamera verwendet,\n\
        die zweite als rechte virtuelle Kamera.",
    ));
    desc.set_use_markup(true);
    desc.set_xalign(0.0);
    tab_box.append(&desc);

    tab_box.append(&Separator::new(Orientation::Horizontal));

    let folder_box = Box::new(Orientation::Horizontal, 5);
    let folder_entry = Entry::new();
    folder_entry.set_text("recordings");
    folder_entry.set_hexpand(true);
    folder_box.append(&folder_entry);

    let load_btn = Button::with_label("Laden");
    load_btn.add_css_class("suggested-action");
    folder_box.append(&load_btn);

    tab_box.append(&folder_box);

    let status_label = Label::new(Some("<i>Nicht geladen</i>"));
    status_label.set_use_markup(true);
    status_label.set_xalign(0.0);
    tab_box.append(&status_label);

    let stereo_system: Rc<RefCell<Option<StereoPlaybackSystem>>> = Rc::new(RefCell::new(None));

    let folder_entry_clone = folder_entry.clone();
    let status_label_clone = status_label.clone();
    let stereo_system_clone = stereo_system.clone();

    load_btn.connect_clicked(move |_| {
        let folder_path = PathBuf::from(folder_entry_clone.text().as_str());

        match StereoPlaybackSystem::load_from_directory(&folder_path) {
            Ok(system) => {
                let status = system.get_status();
                status_label_clone.set_markup(&format!("<b>Geladen:</b>\n{}", status));
                *stereo_system_clone.borrow_mut() = Some(system);
            }
            Err(e) => {
                status_label_clone
                    .set_markup(&format!("<span foreground='red'>Fehler: {}</span>", e));
            }
        }
    });

    tab_box
}

fn create_playback_tab() -> Box {
    let tab_box = Box::new(Orientation::Vertical, 10);
    tab_box.set_margin_start(10);
    tab_box.set_margin_end(10);
    tab_box.set_margin_top(10);
    tab_box.set_margin_bottom(10);

    let title = Label::new(Some("<b>Aufnahmen wiedergeben</b>"));
    title.set_use_markup(true);
    title.set_xalign(0.0);
    tab_box.append(&title);

    let list_box = Box::new(Orientation::Horizontal, 5);

    let recordings_combo = ComboBoxText::new();
    recordings_combo.append(Some("none"), "Keine Aufnahmen");
    recordings_combo.set_active(Some(0));
    recordings_combo.set_hexpand(true);
    list_box.append(&recordings_combo);

    let refresh_btn = Button::with_label("Aktualisieren");
    let recordings_combo_clone = recordings_combo.clone();
    refresh_btn.connect_clicked(move |_| {
        recordings_combo_clone.remove_all();

        match list_recordings(&PathBuf::from("recordings")) {
            Ok(recordings) => {
                if recordings.is_empty() {
                    recordings_combo_clone.append(Some("none"), "Keine Aufnahmen");
                } else {
                    for rec in recordings {
                        recordings_combo_clone.append(Some(&rec), &rec);
                    }
                }
            }
            Err(e) => {
                println!("Fehler beim Laden der Aufnahmen: {}", e);
                recordings_combo_clone.append(Some("none"), "Fehler beim Laden");
            }
        }
        recordings_combo_clone.set_active(Some(0));
    });
    list_box.append(&refresh_btn);

    tab_box.append(&list_box);

    let play_btn = Button::with_label("Abspielen");
    play_btn.add_css_class("suggested-action");
    tab_box.append(&play_btn);

    tab_box
}
