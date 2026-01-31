use eframe::egui;
use crate::cpu::controller::SimulatorController;

pub struct DebugSimulation;

impl DebugSimulation {
    pub fn ui(ctx: &egui::Context, ctrl: &SimulatorController) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Debug simulator");

            ui.horizontal(|ui| {
                if ui.button("Step").clicked() {
                    ctrl.step();
                }
                if ui.button("Run").clicked() {
                    ctrl.run();
                }
                if ui.button("Reset").clicked() {
                    ctrl.reset();
                }
            });
        });
    }
}


// use eframe::egui;
//
// use crate::cpu::Cpu;
// use super::screen::Screen;
// use crate::cpu::controller::SimulatorController;
//
// pub struct DebugSimApp {
//     controller: SimulatorController,
//     screen: Screen,
// }
//
// impl DebugSimApp {
//     pub fn new(cpu: Cpu) -> Self {
//         Self {
//             controller: SimulatorController::new(cpu),
//             screen: Screen::default(),
//         }
//     }
// }
//
// impl eframe::App for DebugSimApp {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             self.screen.ui(ui);
//
//             ui.separator();
//
//             ui.horizontal(|ui| {
//                 if ui.button("Step").clicked() {
//                     self.controller.step();
//                 }
//                 if ui.button("Run").clicked() {
//                     self.controller.run();
//                 }
//                 if ui.button("Reset").clicked() {
//                     self.controller.reset();
//                 }
//             });
//         });
//     }
// }
