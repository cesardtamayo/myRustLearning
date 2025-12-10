use anyhow::Result;
use eframe::egui;
use egui::{Button, Color32, RichText, TextBuffer, TextEdit, Ui};
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;
mod serial;
use crate::serial::*;
/*
 *
 * Find connected taser 10 by sending "m_ver" to all connected devices.
 * h v
 * h 3
 * The T10 will return "mayor.minor.patch" string.
 *
*/

const APP_NAME: &str = "T10 Detector";
const WINDOW_WIDTH: f32 = 400.0;
const WINDOW_HEIGHT: f32 = 400.0;
const BUTTON_SIZE: f32 = 18.0;
const HEADING_FONT_SIZE: f32 = 25.0;
const LABEL_FONT_SIZE: f32 = 15.0;
const BIGGER_SPACING_SIZE: f32 = 10.0;
const SMALLER_SPACING_SIZE: f32 = 5.0;

#[derive(Default)]
struct EguiApp {
    state: Arc<Mutex<AppState>>,
    port_name: Arc<Mutex<String>>,
    serial_response: Arc<Mutex<String>>,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
enum AppState {
    #[default]
    DeviceScanState,
    SerialComState,
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(WINDOW_WIDTH, WINDOW_HEIGHT)),
        ..Default::default()
    };
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| Ok(Box::new(EguiApp::new(cc)))),
    )
    .expect("Failed to app");
}

impl EguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
    fn get_state(&self) -> AppState {
        self.state.lock().clone()
    }
    fn change_state(&mut self, new_state: AppState) {
        let mut guard = self.state.lock();
        (*guard) = new_state;
    }
    fn get_port_name(&self) -> String {
        self.port_name.lock().clone()
    }
    fn change_port_name(&mut self, new_port_name: String) {
        let mut guard = self.port_name.lock();
        (*guard) = new_port_name;
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let current_state = self.get_state();

        let result = match current_state {
            AppState::DeviceScanState => show_dev_scan_screen(self, ctx),
            AppState::SerialComState => show_serial_com_screen(self, ctx),
        };
        if result.is_err() {
            println!("Error occured!")
        }
    }
}

fn show_dev_scan_screen(app: &mut EguiApp, ctx: &egui::Context) -> Result<()> {
    egui::CentralPanel::default().show(ctx, |ui| -> Result<()> {
        get_dev_scan_screen(app, ui)?;
        Ok(())
    });
    Ok(())
}

fn show_serial_com_screen(app: &mut EguiApp, ctx: &egui::Context) -> Result<()> {
    egui::CentralPanel::default().show(ctx, |ui| -> Result<()> {
        get_serial_com_screen(app, ui)?;
        Ok(())
    });
    Ok(())
}

fn get_dev_scan_screen(app: &mut EguiApp, ui: &mut Ui) -> Result<()> {
    ui.heading(RichText::new(APP_NAME).size(HEADING_FONT_SIZE).strong());
    ui.add_space(BIGGER_SPACING_SIZE);

    ui.add_space(SMALLER_SPACING_SIZE);

    if ui
        .add(Button::new(RichText::new("Scan devices").size(BUTTON_SIZE)))
        .clicked()
    {
        // 1. Clone the appState mutex so it can be when T10 is detected
        let port_name_arc_clone = app.port_name.clone(); // The Arc<Mutex<String>>
        let state_arc_clone = app.state.clone(); // The Arc<Mutex<AppState>>
        let ctx_clone = ui.ctx().clone();
        thread::spawn(move || {
            loop {
                let state_arc_iteration_clone = state_arc_clone.clone();
                let port_name_iteration_clone = port_name_arc_clone.clone();
                if find_t10(state_arc_iteration_clone, port_name_iteration_clone) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
            ctx_clone.request_repaint();
        });
        // ui.ctx().request_repaint();
    }

    Ok(())
}

fn get_serial_com_screen(app: &mut EguiApp, ui: &mut Ui) -> Result<()> {
    ui.heading(RichText::new(APP_NAME).size(HEADING_FONT_SIZE).strong());
    ui.add_space(BIGGER_SPACING_SIZE);
    ui.label(
        RichText::new("T10 Detected!".to_string())
            .size(LABEL_FONT_SIZE)
            .color(Color32::GREEN),
    );

    ui.add_space(SMALLER_SPACING_SIZE);
    ui.label(
        RichText::new("Port Name".to_string())
            .size(LABEL_FONT_SIZE)
            .color(Color32::GREEN),
    );
    let mut port_guard = app.port_name.lock();
    let current_port_name: &mut String = &mut *port_guard;
    ui.add_sized(
        [ui.available_width(), 20.0],
        TextEdit::singleline(current_port_name),
    );
    drop(port_guard);

    ui.add_space(SMALLER_SPACING_SIZE);
    ui.label(
        RichText::new("Serial Response".to_string())
            .size(LABEL_FONT_SIZE)
            .color(Color32::GREEN),
    );
    let mut response_guard = app.serial_response.lock();
    let current_response: &mut String = &mut *response_guard;
    ui.add_sized(
        [ui.available_width(), 20.0],
        TextEdit::singleline(current_response),
    );
    drop(response_guard);

    if ui
        .add(Button::new(RichText::new("Back").size(BUTTON_SIZE)))
        .clicked()
    {
        app.change_state(AppState::DeviceScanState);
    }

    if ui
        .add(Button::new(
            RichText::new("Get FW Version").size(BUTTON_SIZE),
        ))
        .clicked()
    {
        println!("Getting FW version ...");
        let port_name_clone = app.port_name.clone();
        let serial_response_clone = app.serial_response.clone();
        let ctx_clone = ui.ctx().clone();
        thread::spawn(move || {
            send_serial_command(port_name_clone, "m ver", serial_response_clone);
            ctx_clone.request_repaint();
        });
    }

    if ui
        .add(Button::new(
            RichText::new("Get HV version").size(BUTTON_SIZE),
        ))
        .clicked()
    {
        println!("Getting hv version ...");
        // send_serial_command(&app.serial_response, "h v");
        let port_name_clone = app.port_name.clone();
        let serial_response_clone = app.serial_response.clone();
        let ctx_clone = ui.ctx().clone();
        thread::spawn(move || {
            send_serial_command(port_name_clone, "h c", serial_response_clone);
            ctx_clone.request_repaint();
        });
    }

    if ui
        .add(Button::new(
            RichText::new("Get HV Active Bank").size(BUTTON_SIZE),
        ))
        .clicked()
    {
        println!("Getting hv active bank ...");
        // send_serial_command(&app.serial_response, "h v");
        let port_name_clone = app.port_name.clone();
        let serial_response_clone = app.serial_response.clone();
        let ctx_clone = ui.ctx().clone();
        thread::spawn(move || {
            send_serial_command(port_name_clone, "h 3", serial_response_clone);
            ctx_clone.request_repaint();
        });
    }

    Ok(())
}
