use eframe::egui;

pub const COLS: usize = 80;
pub const ROWS: usize = 80;

pub struct Screen {
    buffer: [[char; COLS]; ROWS],
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            buffer: [[' '; COLS]; ROWS],
        }
    }
}

impl Screen {
    pub fn ui(&self, ui: &mut egui::Ui) {
        let text: String = self.buffer
            .iter()
            .map(|r| r.iter().collect::<String>() + "\n")
            .collect();

        ui.add(
            egui::TextEdit::multiline(&mut text.as_str())
                .font(egui::TextStyle::Monospace)
                .desired_rows(ROWS)
                .interactive(false),
        );
    }
}
