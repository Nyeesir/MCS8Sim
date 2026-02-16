use iced::{border, window, Element, Length, Task, Theme};
use iced::widget::{container, column, scrollable, text};

use crate::cpu::InstructionTrace;
use crate::gui::preferences::WindowGeometry;

const WINDOW_WIDTH: f32 = 520.0;
const WINDOW_HEIGHT: f32 = 340.0;

pub fn open_window() -> (window::Id, Task<window::Id>) {
    open_window_with_geometry(None)
}

pub fn open_window_with_geometry(
    geometry: Option<WindowGeometry>,
) -> (window::Id, Task<window::Id>) {
    let mut settings = window::Settings {
        size: iced::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        min_size: Some(iced::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
        max_size: Some(iced::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
        ..window::Settings::default()
    };
    if let Some(geometry) = geometry {
        geometry.apply_to_settings(&mut settings);
    }
    window::open(settings)
}


pub fn view<'a, Message: 'a>(
    entries: &'a [InstructionTrace],
) -> Element<'a, Message> {
    let body = if entries.is_empty() {
        column![text("No instructions yet.").font(iced::Font::MONOSPACE)]
    } else {
        let last_index = entries.len().saturating_sub(1);
        let lines = entries.iter().enumerate().map(|(idx, entry)| {
            let label = format!("Addr: {:#06X} | Instr: {}", entry.address, entry.text);
            let line = text(label).font(iced::Font::MONOSPACE);
            if idx == last_index {
                container(line)
                    .padding(4)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        container::Style::default()
                            .background(palette.primary.weak.color)
                            .border(border::rounded(3).color(palette.primary.strong.color))
                    })
                    .width(Length::Fill)
                    .into()
            } else {
                container(line)
                    .padding(2)
                    .width(Length::Fill)
                    .into()
            }
        });
        column(lines.collect::<Vec<_>>()).spacing(4)
    };

    let content = scrollable(container(body).padding(10))
        .width(Length::Fill)
        .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
