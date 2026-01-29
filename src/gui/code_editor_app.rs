use std::fs;

use eframe::egui;
use egui_code_editor::{CodeEditor as EguiCodeEditor, ColorTheme, Syntax};

use crate::assembler::Assembler;
use crate::cpu::Cpu;
use crate::gui::common::apply_global_style;
use crate::gui::simulator_launcher::{launch_basic, launch_debug};

pub struct EditorApp {
    code: String,
    editor: EguiCodeEditor,
    asm_error: Option<String>,
}

//TODO: PODSWIETLANIE WADLIWEJ LINIJKI JEZELI MOZLIWE
//TODO: SYNTAX KOLORKI
//TODO: OGARNAC ROZMIAR CZCIONKI
//FIXME: OKNO SYMULATORA NIE WYSWIETLA SIE, NIC NIE DZIALA

impl Default for EditorApp {
    fn default() -> Self {
        let editor = EguiCodeEditor::default()
            .id_source("source_editor")
            .with_numlines(true)
            .with_fontsize(16.0)
            .with_syntax(Syntax::asm())
            .with_theme(ColorTheme::GITHUB_DARK);

        Self {
            code: "ORG 800h\n".into(),
            editor,
            asm_error: None,
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_global_style(ctx);

        // ===== BOTTOM BAR =====
        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Load file").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        if let Ok(content) = fs::read_to_string(path) {
                            self.code = content;
                            self.asm_error = None;
                        }
                    }
                }

                ui.separator();

                if ui.button("Run simulation").clicked() {
                    self.run(false);
                }

                if ui.button("Run simulation with debug tools").clicked() {
                    self.run(true);
                }
            });
        });

        // ===== ERROR PANEL =====
        if let Some(err) = &self.asm_error {
            egui::TopBottomPanel::bottom("error_panel").show(ctx, |ui| {
                ui.colored_label(egui::Color32::RED, err);
            });
        }

        // ===== EDITOR =====
        egui::CentralPanel::default().show(ctx, |ui| {
            self.editor.show(ui, &mut self.code);
        });
    }
}

impl EditorApp {
    fn run(&mut self, debug: bool) {
        match Assembler::new().assemble(&self.code) {
            Ok(memory) => {
                let cpu = Cpu::with_memory(memory);
                self.asm_error = None;
                if debug {
                    launch_debug(cpu);
                } else {
                    launch_basic(cpu);
                }
            }
            Err(e) => self.asm_error = Some(e.to_string()),
        }
    }
}
