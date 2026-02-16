use iced::{window, Element, Length, Task};
use iced::widget::{container, column, row, text};
use crate::cpu::CpuState;
use crate::gui::preferences::WindowGeometry;

const WINDOW_WIDTH: f32 = 560.0;
const WINDOW_HEIGHT: f32 = 320.0;

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



fn flag_char(bit: bool, ch: char) -> char {
    if bit { ch } else { '.' }
}

fn format_flags_bin(flags: u8) -> String {
    let mut s = String::with_capacity(8);
    for i in (0..8).rev() {
        let bit = (flags >> i) & 1;
        s.push(if bit == 1 { '1' } else { '0' });
    }
    s
}

const COL_REG_W: f32 = 52.0;
const COL_HEX_W: f32 = 62.0;
const COL_DEC_W: f32 = 58.0;
const COL_OCT_W: f32 = 58.0;
const COL_BIN_W: f32 = 86.0;
const COL_ASCII_W: f32 = 70.0;

fn format_bin8(value: u8) -> String {
    let mut s = String::with_capacity(8);
    for i in (0..8).rev() {
        let bit = (value >> i) & 1;
        s.push(if bit == 1 { '1' } else { '0' });
    }
    s
}

fn ascii_char(value: u8) -> char {
    if (0x20..=0x7E).contains(&value) {
        value as char
    } else {
        ' '
    }
}

fn header_cell<'a, Message: 'a>(label: &'a str, width: f32) -> iced::Element<'a, Message> {
    text(label)
        .width(Length::Fixed(width))
        .font(iced::Font::MONOSPACE)
        .into()
}

fn value_cell<'a, Message: 'a>(value: String, width: f32) -> iced::Element<'a, Message> {
    text(value)
        .width(Length::Fixed(width))
        .font(iced::Font::MONOSPACE)
        .into()
}

fn row_reg<'a, Message: 'a>(label: &'a str, value: u8) -> iced::Element<'a, Message> {
    row![
        value_cell(label.to_string(), COL_REG_W),
        value_cell(format!("{:#04X}", value), COL_HEX_W),
        value_cell(format!("{}", value), COL_DEC_W),
        value_cell(format!("{:03o}", value), COL_OCT_W),
        value_cell(format_bin8(value), COL_BIN_W),
        value_cell(format!("'{}'", ascii_char(value)), COL_ASCII_W),
    ]
    .spacing(8)
    .into()
}

pub fn view<'a, Message: 'a>(
    state: &CpuState,
) -> Element<'a, Message> {
    let flags = state.flags;
    let flags_str = [
        flag_char(flags & 0x80 != 0, 'S'),
        flag_char(flags & 0x40 != 0, 'Z'),
        flag_char(flags & 0x10 != 0, 'A'),
        flag_char(flags & 0x04 != 0, 'P'),
        flag_char(flags & 0x01 != 0, 'C'),
    ]
    .iter()
    .collect::<String>();

    let regs = column![
        row![
            header_cell("REG", COL_REG_W),
            header_cell("HEX", COL_HEX_W),
            header_cell("DEC", COL_DEC_W),
            header_cell("OCT", COL_OCT_W),
            header_cell("BIN", COL_BIN_W),
            header_cell("ASCII", COL_ASCII_W),
        ]
        .spacing(8),
        row_reg("A", state.a),
        row_reg("B", state.b),
        row_reg("C", state.c),
        row_reg("D", state.d),
        row_reg("E", state.e),
        row_reg("H", state.h),
        row_reg("L", state.l),
        row![
            header_cell("FLAGS", COL_REG_W),
            value_cell(String::new(), COL_HEX_W),
            value_cell(String::new(), COL_DEC_W),
            value_cell(String::new(), COL_OCT_W),
            value_cell(format_flags_bin(flags), COL_BIN_W),
            value_cell(format!("[{}]", flags_str), COL_ASCII_W),
        ]
        .spacing(8),
        row![
            header_cell("SP", COL_REG_W),
            value_cell(format!("{:#06X}", state.stack_pointer), COL_HEX_W),
            value_cell(format!("{}", state.stack_pointer), COL_DEC_W),
            value_cell(String::new(), COL_OCT_W),
            value_cell(String::new(), COL_BIN_W),
            value_cell(String::new(), COL_ASCII_W),
        ]
        .spacing(8),
        row![
            header_cell("PC", COL_REG_W),
            value_cell(format!("{:#06X}", state.program_counter), COL_HEX_W),
            value_cell(format!("{}", state.program_counter), COL_DEC_W),
            value_cell(String::new(), COL_OCT_W),
            value_cell(String::new(), COL_BIN_W),
            value_cell(String::new(), COL_ASCII_W),
        ]
        .spacing(8),
    ]
    .spacing(6);

    container(regs)
        .padding(12)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
