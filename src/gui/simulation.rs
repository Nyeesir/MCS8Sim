use iced::{window, Element, Length, Task};
use iced::widget::{button, container, row, scrollable, text};

const CHAR_WIDTH_PX: f32 = 8.0;
const CHAR_HEIGHT_PX: f32 = 16.0;
const CONSOLE_COLS: f32 = 80.0;
const CONSOLE_ROWS: f32 = 40.0;
const CONSOLE_PADDING: f32 = 16.0;

//TODO: NEED TO CHECK IF \R AND \N WORKS AS INTENDED

pub fn open_window() -> (window::Id, Task<window::Id>) {
    let width = (CONSOLE_COLS * CHAR_WIDTH_PX) + (CONSOLE_PADDING * 2.0);
    let height = (CONSOLE_ROWS * CHAR_HEIGHT_PX) + (CONSOLE_PADDING * 2.0);
    window::open(window::Settings {
        size: iced::Size::new(width, height),
        min_size: Some(iced::Size::new(width, height)),
        max_size: Some(iced::Size::new(width, height)),
        ..window::Settings::default()
    })
}

pub fn view<'a, Message: 'a + Clone>(
    output: &'a str,
    waiting_for_input: bool,
    debug_mode: bool,
    cycles_per_second: u64,
    is_halted: bool,
    start: Message,
    stop: Message,
    reset: Message,
    step: Message,
) -> Element<'a, Message> {
    let indicator = if waiting_for_input {
        text("Waiting for input...")
    } else {
        text("")
    };

    let step_button: Element<'a, Message> = if debug_mode {
        button("Step").on_press(step).into()
    } else {
        iced::widget::Space::new()
            .width(Length::Shrink)
            .height(Length::Shrink)
            .into()
    };

    let controls = row![
        button("Start").on_press(start),
        button("Stop").on_press(stop),
        button("Reset").on_press(reset),
        step_button,
        iced::widget::Space::new().width(Length::Fill),
        indicator,
    ]
    .spacing(8);

    let content = text(output).font(iced::Font::MONOSPACE).size(14);
    let output_view = scrollable(container(content).padding(CONSOLE_PADDING))
        .width(Length::Fill)
        .height(Length::Fill);

    let mut footer_text = format!("Cycles/sec: {}", cycles_per_second);
    if is_halted {
        footer_text.push_str(" | CPU HALTED");
    }
    let footer = container(
        text(footer_text)
            .size(14)
            .font(iced::Font::MONOSPACE),
    )
    .padding(4);

    container(
        iced::widget::column![
            controls,
            output_view,
            footer
        ]
            .spacing(8),
    )
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
