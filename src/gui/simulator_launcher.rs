use crate::cpu::Cpu;
use super::{basic_simulation_app::BasicSimApp, debug_simulation_app::DebugSimApp};

pub fn launch_basic(cpu: Cpu) {
    std::thread::spawn(move || {
        let options = eframe::NativeOptions::default();

        let _ = eframe::run_native(
            "MCS-8 Simulator",
            options,
            Box::new(|_cc| Ok(Box::new(BasicSimApp::new(cpu)))),
        );
    });
}


pub fn launch_debug(cpu: Cpu) {
    std::thread::spawn(move || {
        let options = eframe::NativeOptions::default();

        let _ = eframe::run_native(
            "8080 Simulator (Debug)",
            options,
            Box::new(|_cc| Ok(Box::new(DebugSimApp::new(cpu)))),
        );
    });
}