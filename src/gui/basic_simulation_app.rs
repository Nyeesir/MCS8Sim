use eframe::egui;

use crate::cpu::Cpu;
use super::screen::Screen;
use crate::cpu::controller::SimulatorController;

pub struct BasicSimApp {
    controller: SimulatorController,
    screen: Screen,
}

impl BasicSimApp {
    pub fn new(cpu: Cpu) -> Self {
        Self {
            controller: SimulatorController::new(cpu),
            screen: Screen::default(),
        }
    }
}

impl eframe::App for BasicSimApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.screen.ui(ui);

            ui.separator();

            if ui.button("Run").clicked() {
                self.controller.run();
            }
        });
    }
}
