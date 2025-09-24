pub struct CodeEditor{
    code: String
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            code: "ORG 800h".into()
        }
    }
}

impl eframe::App for CodeEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Source code editor");
            ui.horizontal(|ui| {
                ui.code_editor(&mut self.code)
            });
        });
    }
}