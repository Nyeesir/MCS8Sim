use std::fs;

use eframe::egui;
use egui_code_editor::{CodeEditor as EguiCodeEditor, ColorTheme, Syntax};

use crate::assembler::Assembler;
use crate::cpu::Cpu;
use crate::cpu::controller::SimulatorController;
use crate::assembler::errors::AssemblyError;

const MIN_FONT_SIZE: f32 = 8.0;
const MAX_FONT_SIZE: f32 = 64.0;

//TODO: PODSWIETLANIE WADLIWEJ LINIJKI JEZELI MOZLIWE
//TODO: SYNTAX KOLORKI
//TODO: OGARNAC ROZMIAR CZCIONKI
//TODO: MOZE JEZELI NIE MOZE WCZYTAC BIOSU TO PRZYCISK DO WYBRANIA PLIKU??
pub struct CodeEditor {
    code: String,
    editor: EguiCodeEditor,
    error: Option<String>,
    load_bios: bool,
    asm_error: Option<AssemblyError>,
    font_size: f32
}

impl Default for CodeEditor {
    fn default() -> Self {
        let font_size = 14f32;

        let mut editor = EguiCodeEditor::default()
            .id_source("source_editor")
            .with_numlines(true)
            .with_syntax(Syntax::asm())
            .with_fontsize(font_size)
            .with_theme(ColorTheme::GITHUB_DARK);

        Self {
            code: "ORG 800h\n".into(),
            editor,
            error: None,
            load_bios: true,
            asm_error: None,
            font_size
        }
    }
}

impl CodeEditor {
    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        controller: &mut Option<SimulatorController>,
        show_basic: &mut bool,
        show_debug: &mut bool,
    ) {
        egui::TopBottomPanel::bottom("editor_bar").frame(egui::Frame::default().outer_margin(egui::Margin::same(8))).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Load file").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        if let Ok(content) = fs::read_to_string(path) {
                            self.code = content;
                            self.error = None;
                        }
                    }
                }

                if ui.button("Run simulation").clicked() {
                    self.run(false, controller, show_basic, show_debug);
                }

                if ui.button("Run simulation with debug tools").clicked() {
                    self.run(true, controller, show_basic, show_debug);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.checkbox(&mut self.load_bios, "Load BIOS");
                })
            });
        });

        if let Some(err) = &self.error {
            egui::TopBottomPanel::bottom("error_panel").show(ctx, |ui| {
                ui.colored_label(egui::Color32::RED, err);
            });
        }

        egui::SidePanel::right("right_panel")
            .resizable(false)
            .default_width(120.0)
            .frame(egui::Frame::NONE.inner_margin(egui::Margin::same(8)))
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center),|ui| {
                    let width = ui.available_width();

                    let response = ui.add_sized(
                        [width, 0.0],
                        egui::DragValue::new(&mut self.font_size)
                            .range(MIN_FONT_SIZE..=MAX_FONT_SIZE)
                            .speed(0.5)
                            .prefix("Font: "),
                    );

                    if response.changed() {
                        self.rebuild_editor();
                    }

                    let width = ui.available_width();

                    if ui.add_sized([width, 0.0], egui::Button::new("Font -")).clicked(){
                        self.font_size = (self.font_size - 1.0).clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
                        self.rebuild_editor();
                    }

                    if ui.add_sized([width, 0.0], egui::Button::new("Font +")).clicked(){
                        self.font_size = (self.font_size + 1.0).clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
                        self.rebuild_editor();
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let editor = egui::TextEdit::multiline(&mut self.code)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    .desired_rows(10)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY);
                    // .layouter(&mut layouter);
                let editor = if cfg!(feature = "syntect") {
                    editor
                } else {
                    use egui::Color32;
                    let background_color = Color32::BLACK;
                    editor.background_color(background_color)
                };
                ui.add(editor);
            });

            // ui.add(egui::TextEdit::multiline(&mut self.code)
            //     .font(egui::TextStyle::Monospace)
            //     .code_editor()
            //     .desired_rows(10)
            //     .lock_focus(true)
            //     .desired_width(f32::INFINITY)
            //     .desired_width(ui.available_width()));
            // let editor_response = ui.allocate_ui(
            //     ui.available_size(),
            //     |ui| {
            //         self.editor.show(ui, &mut self.code);
            //     },
            // );
            //
            // if editor_response.response.has_focus() {
            //     ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
            // }
        });
    }

    fn run(
        &mut self,
        debug: bool,
        controller: &mut Option<SimulatorController>,
        show_basic: &mut bool,
        show_debug: &mut bool,
    ) {
        let mut mem = [0; (u16::MAX as usize) + 1];

        if self.load_bios {
            if let Err(e) = load_bios(&mut mem) {
                self.error = Some(e);
                return;
            }
        }

        match Assembler::new().assemble(&self.code) {
            Ok(program_mem) => {
                self.asm_error = None; // 👈 WYCZYŚĆ
                self.error = None;

                let first_non_zero = program_mem.iter().position(|&v| v != 0);
                let last_non_zero = program_mem.iter().rposition(|&v| v != 0);

                if let (Some(start), Some(end)) = (first_non_zero, last_non_zero) {
                    mem[start..=end].copy_from_slice(&program_mem[start..=end]);
                }

                let cpu = Cpu::with_memory(mem);
                *controller = Some(SimulatorController::new(cpu));
                *show_basic = !debug;
                *show_debug = debug;
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.asm_error = Some(e);
            },

        }
    }

    fn rebuild_editor(&mut self) {
        self.editor = EguiCodeEditor::default()
            .id_source("source_editor")
            .with_numlines(true)
            .with_syntax(Syntax::asm())
            .with_fontsize(self.font_size)
            .with_theme(ColorTheme::GITHUB_DARK);
    }
}

fn load_bios(mem: &mut [u8]) -> Result<(), String> {
    let bios = std::fs::read("src/bios.bin")
        .map_err(|e| format!("Failed to load BIOS: {e}"))?;

    if bios.len() > mem.len() {
        return Err(format!(
            "BIOS too large: {} bytes (max {})",
            bios.len(),
            mem.len()
        ));
    }

    mem[..bios.len()].copy_from_slice(&bios);
    Ok(())
}


// fn draw_error_line_highlight(
//     ui: &egui::Ui,
//     editor_rect: egui::Rect,
//     error: &AssemblyError,
// ) {
//     let line = error.line_number.saturating_sub(1);
//
//     let row_height = 14f32;
//     let y = editor_rect.top() + row_height * line as f32;
//
//     let rect = egui::Rect::from_min_max(
//         egui::pos2(editor_rect.left(), y),
//         egui::pos2(editor_rect.right(), y + row_height),
//     );
//
//     ui.painter().rect_filled(
//         rect,
//         0.0,
//         egui::Color32::from_rgba_unmultiplied(255, 0, 0, 60),
//     );
// }
