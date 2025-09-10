pub mod assembler;
#[cfg(test)] 
mod tests;
pub mod cpu;
pub mod gui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "MCS-8 Simulator",
        options,
        Box::new(|cc| {
            Ok(Box::<crate::gui::code_editor::CodeEditor>::default())
        }),
    )
}