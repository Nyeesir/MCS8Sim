pub mod assembler;
#[cfg(test)] 
mod tests;
pub mod cpu;
pub mod gui;
pub mod deassembler;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "MCS-8 Simulator",
        options,
        Box::new(|cc| {
            Ok(Box::<crate::gui::main_app::MainApp>::default())
        }),
    )
}