use std::fmt::Write;

use iced::advanced::text::LineHeight;
use iced::{window, Element, Length, Task};
use iced::widget::{container, scrollable, text, Id};

use crate::gui::preferences::WindowGeometry;

const FONT_SIZE: f32 = 18.0;
const WINDOW_WIDTH: f32 = FONT_SIZE * 33.0;
const WINDOW_HEIGHT: f32 = FONT_SIZE * 9.0;
const BYTES_PER_ROW: usize = 16;
pub const MEMORY_VIEW_SIZE: usize = (u16::MAX as usize) + 1;
pub const TOTAL_ROWS: usize = MEMORY_VIEW_SIZE / BYTES_PER_ROW;
pub const VISIBLE_ROWS: usize = 8;
pub const ROW_HEIGHT_PX: f32 = 18.0;
pub const CONTENT_PADDING: f32 = 10.0;
pub const MEMORY_SCROLL_ID: &str = "memory_scroll";
const HEADER_TEXT: &str = "       00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F";

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


pub fn format_memory_rows(bytes: &[u8], start_row: usize, row_count: usize) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    let total_rows = (bytes.len() + BYTES_PER_ROW - 1) / BYTES_PER_ROW;
    let start_row = start_row.min(total_rows);
    let end_row = (start_row + row_count).min(total_rows);
    let mut output = String::with_capacity((end_row - start_row) * (6 + BYTES_PER_ROW * 3 + 1));

    for row_idx in start_row..end_row {
        let address = row_idx * BYTES_PER_ROW;
        let start = address.min(bytes.len());
        let end = (start + BYTES_PER_ROW).min(bytes.len());
        let chunk = &bytes[start..end];
        let _ = write!(output, "{:04X}: ", address);
        for (idx, value) in chunk.iter().enumerate() {
            let _ = write!(output, "{:02X}", value);
            if idx + 1 < BYTES_PER_ROW {
                output.push(' ');
            }
        }
        output.push('\n');
    }
    output
}

pub fn view<'a, Message: 'a>(
    memory_text: &'a str,
    start_row: usize,
    on_scroll: impl Fn(scrollable::Viewport) -> Message + 'a,
) -> Element<'a, Message> {
    let body = if memory_text.is_empty() {
        text("No memory snapshot yet.")
            .font(iced::Font::MONOSPACE)
            .size(FONT_SIZE)
    } else {
        text(memory_text)
            .font(iced::Font::MONOSPACE)
            .size(FONT_SIZE)
            .line_height(LineHeight::Absolute(iced::Pixels(ROW_HEIGHT_PX)))
    };

    let top_rows = start_row.min(TOTAL_ROWS);
    let bottom_rows = TOTAL_ROWS.saturating_sub(top_rows + VISIBLE_ROWS);
    let top_space = iced::widget::Space::new()
        .height(Length::Fixed(top_rows as f32 * ROW_HEIGHT_PX))
        .width(Length::Fill);
    let bottom_space = iced::widget::Space::new()
        .height(Length::Fixed(bottom_rows as f32 * ROW_HEIGHT_PX))
        .width(Length::Fill);

    let header = text(HEADER_TEXT)
        .font(iced::Font::MONOSPACE)
        .size(FONT_SIZE)
        .line_height(LineHeight::Absolute(iced::Pixels(ROW_HEIGHT_PX)));

    let content = scrollable(
        container(
            iced::widget::column![top_space, body, bottom_space]
                .width(Length::Fill),
        )
        .padding(CONTENT_PADDING),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .id(Id::new(MEMORY_SCROLL_ID))
    .on_scroll(on_scroll);

    container(iced::widget::column![header, content])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
