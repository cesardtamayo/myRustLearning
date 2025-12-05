use anyhow::Result;
use eframe::egui;
use egui::{Button, Color32, RichText, Ui};

const WINDOW_WIDTH: f32 = 400.0;
const WINDOW_HEIGHT: f32 = 400.0;
const BUTTON_SIZE: f32 = 18.0;
const HEADING_FONT_SIZE: f32 = 25.0;
const LABEL_FONT_SIZE: f32 = 15.0;
const BIGGER_SPACING_SIZE: f32 = 10.0;
const SMALLER_SPACING_SIZE: f32 = 5.0;

#[derive(Default)]
struct EguiApp {
    state: AppState,
}

#[derive(Default, Debug, PartialEq, Eq)]
enum AppState {
    #[default]
    SomeState,
    SomeOtherState,
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(WINDOW_WIDTH, WINDOW_HEIGHT)),
        ..Default::default()
    };
    eframe::run_native(
        "My-Rust-App",
        options,
        Box::new(|cc| Ok(Box::new(EguiApp::new(cc)))),
    )
    .expect("Failed to app");
}

impl EguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
    fn change_state(&mut self, new_state: AppState) {
        self.state = new_state;
    }
}
impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let result = match self.state {
            AppState::SomeState => show_some_screen(self, ctx),
            AppState::SomeOtherState => show_some_other_screen(self, ctx),
        };
        if result.is_err() {
            println!("Error occured!")
        }
    }
}

fn show_some_screen(app: &mut EguiApp, ctx: &egui::Context) -> Result<()> {
    egui::CentralPanel::default().show(ctx, |ui| -> Result<()> {
        get_ui1(app, ui)?;
        Ok(())
    });
    Ok(())
}

fn show_some_other_screen(app: &mut EguiApp, ctx: &egui::Context) -> Result<()> {
    egui::CentralPanel::default().show(ctx, |ui| -> Result<()> {
        get_ui2(app, ui)?;
        Ok(())
    });
    Ok(())
}

fn get_ui1(app: &mut EguiApp, ui: &mut Ui) -> Result<()> {
    ui.heading(
        RichText::new("My Rust App")
            .size(HEADING_FONT_SIZE)
            .strong(),
    );
    ui.add_space(BIGGER_SPACING_SIZE);
    ui.label(
        RichText::new("State1 in Green".to_string())
            .size(LABEL_FONT_SIZE)
            .color(Color32::GREEN),
    );
    ui.add_space(SMALLER_SPACING_SIZE);
    if ui
        .add(Button::new(
            RichText::new("Button to switch states").size(BUTTON_SIZE),
        ))
        .clicked()
    {
        app.change_state(AppState::SomeOtherState);
    }
    Ok(())
}

fn get_ui2(app: &mut EguiApp, ui: &mut Ui) -> Result<()> {
    ui.heading(
        RichText::new("My Rust App")
            .size(HEADING_FONT_SIZE)
            .strong(),
    );
    ui.add_space(BIGGER_SPACING_SIZE);
    ui.label(
        RichText::new("State2 in yellow".to_string())
            .size(LABEL_FONT_SIZE)
            .color(Color32::YELLOW),
    );
    ui.add_space(SMALLER_SPACING_SIZE);

    if ui
        .add(Button::new(
            RichText::new("Button to switch states").size(BUTTON_SIZE),
        ))
        .clicked()
    {
        app.change_state(AppState::SomeState);
    }
    Ok(())
}
