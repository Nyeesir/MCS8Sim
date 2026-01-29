use eframe::egui;

pub fn apply_global_style(ctx: &egui::Context) {
    ctx.set_pixels_per_point(1.4);

    ctx.style_mut(|style| {
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(64.0, egui::FontFamily::Monospace),
        );
    });
}
