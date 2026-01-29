use std::fs;

use eframe::egui;
use egui_code_editor::CodeEditor as EguiCodeEditor;

use crate::cpu::Cpu;
use crate::assembler::Assembler;

pub struct App {
    code: String,
    editor: EguiCodeEditor,

    asm_error: Option<String>,
    cpu: Option<Cpu>,
    show_simulator: bool,
}

impl Default for App {
    fn default() -> Self {
        let editor = EguiCodeEditor::default()
            .id_source("source_editor")
            .with_numlines(true);

        Self {
            code: "ORG 800h\n".into(),
            editor,
            asm_error: None,
            cpu: None,
            show_simulator: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ====== GLOBAL SCALE (opcjonalne) ======
        ctx.set_pixels_per_point(1.25);

        // ====== ERROR PANEL ======
        if let Some(error) = &self.asm_error {
            egui::TopBottomPanel::bottom("error_panel").show(ctx, |ui| {
                ui.add_space(4.0);
                ui.colored_label(
                    egui::Color32::RED,
                    egui::RichText::new("Assembler error").strong(),
                );
                ui.label(
                    egui::RichText::new(error)
                        .monospace(),
                );
                ui.add_space(4.0);
            });
        }

        // ====== BOTTOM BAR ======
        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Load file").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Assembly", &["asm", "txt"])
                        .pick_file()
                    {
                        if let Ok(content) = fs::read_to_string(path) {
                            self.code = content;
                            self.asm_error = None;
                        }
                    }
                }

                ui.separator();

                if ui.button("Run simulation").clicked() {
                    self.asm_error = None;
                    self.cpu = None;
                    self.show_simulator = false;

                    match Assembler::new().assemble(&self.code) {
                        Ok(memory) => {
                            self.cpu = Some(Cpu::with_memory(memory));
                            self.show_simulator = true;
                        }
                        Err(err) => {
                            self.asm_error = Some(err.to_string());
                        }
                    }
                }

                if ui.button("Run simulation with debug tools").clicked() {
                    // na razie puste
                }
            });
        });

        // ====== MAIN EDITOR ======
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Source code editor");
            ui.separator();

            self.editor.show(ui, &mut self.code);
        });

        // ====== SIMULATOR WINDOW ======
        if self.show_simulator {
            egui::Window::new("Intel 8080 Simulator")
                .resizable(true)
                .collapsible(false)
                .show(ctx, |ui| {
                    if let Some(cpu) = &mut self.cpu {
                        simulator_ui(ui, cpu);
                    }
                });
        }
    }
}

// ====== UI SYMULATORA ======
fn simulator_ui(ui: &mut egui::Ui, cpu: &mut Cpu) {
    ui.heading("CPU Control");
    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Step").clicked() {
            cpu.step();
        }

        if ui.button("Run").clicked() {
            cpu.run();
        }
    });

    ui.separator();
}