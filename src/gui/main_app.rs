use eframe::egui;

use crate::cpu::controller::SimulatorController;
use super::basic_simulation::BasicSimulation;
use super::debug_simulation::DebugSimulation;
use super::code_editor::CodeEditor;

pub struct MainApp {
    editor: CodeEditor,

    controller: Option<SimulatorController>,

    show_basic: bool,
    show_debug: bool,
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            editor: CodeEditor::default(),
            controller: None,
            show_basic: false,
            show_debug: false,
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ===== EDITOR (zawsze widoczny) =====
        self.editor.ui(ctx, &mut self.controller, &mut self.show_basic, &mut self.show_debug);

        // ===== BASIC SIMULATOR VIEWPORT =====
        if self.show_basic {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("basic_sim"),
                egui::ViewportBuilder::default()
                    .with_title("MCS-8 Simulator")
                    .with_inner_size([640.0, 640.0]),
                |ctx, class| {
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_basic = false;
                        return;
                    }

                    if let Some(ctrl) = &self.controller {
                        BasicSimulation::ui(ctx, ctrl);
                    }
                },
            );
        }

        // ===== DEBUG SIMULATOR VIEWPORT =====
        if self.show_debug {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("debug_sim"),
                egui::ViewportBuilder::default()
                    .with_title("MCS-8 Simulator (Debug)")
                    .with_inner_size([640.0, 640.0]),
                |ctx, class| {
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_debug = false;
                        return;
                    }

                    if let Some(ctrl) = &self.controller {
                        DebugSimulation::ui(ctx, ctrl);
                    }
                },
            );
        }
    }
}
