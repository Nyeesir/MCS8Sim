use iced::{window, Element, Length, Task};
use iced::widget::{button, container, row, scrollable, text, text_input};

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
    is_running: bool,
    cycles_limit_input: &'a str,
    on_cycles_limit_input: impl Fn(String) -> Message + 'a,
    on_cycles_limit_submit: Message,
    open_registers: Message,
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
        button("Step").on_press(step).width(Length::Fill).into()
    } else {
        iced::widget::Space::new()
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
    };

    let controls = row![
        iced::widget::Space::new().width(Length::Fill),
        indicator,
    ]
    .spacing(8);

    let content = text(output).font(iced::Font::MONOSPACE).size(14);
    let output_view = scrollable(container(content).padding(CONSOLE_PADDING))
        .width(Length::Fill)
        .height(Length::Fill);

    let state_label = if is_halted {
        "HALTED"
    } else if is_running {
        "RUNNING"
    } else {
        "PAUSED"
    };
    let mut footer_text = format!("Cycles/sec: {} | State: {}", cycles_per_second, state_label);
    if is_halted {
        footer_text.push_str(" | CPU HALTED");
    }
    let limit_input: Element<'a, Message> = if debug_mode {
        text_input("Limit", cycles_limit_input)
            .on_input(on_cycles_limit_input)
            .on_submit(on_cycles_limit_submit)
            .width(Length::Fixed(80.0))
            .into()
    } else {
        iced::widget::Space::new()
            .width(Length::Shrink)
            .height(Length::Shrink)
            .into()
    };

    let footer = container(
        row![
            text(footer_text)
                .size(14)
                .font(iced::Font::MONOSPACE),
            iced::widget::Space::new().width(Length::Fill),
            limit_input
        ]
        .align_y(iced::alignment::Vertical::Center)
        .spacing(8),
    )
    .padding(4);

    let right_panel = {
        let registers_button: Element<'a, Message> = if debug_mode {
            button("Registers").on_press(open_registers).width(Length::Fill).into()
        } else {
            iced::widget::Space::new()
                .width(Length::Shrink)
                .height(Length::Shrink)
                .into()
        };

        container(
            iced::widget::column![
                button("Start").on_press(start).width(Length::Fill),
                button("Stop").on_press(stop).width(Length::Fill),
                button("Reset").on_press(reset).width(Length::Fill),
                step_button,
                iced::widget::Space::new().height(Length::Fill),
                registers_button,
            ]
            .spacing(8),
        )
        .padding(8)
        .width(Length::Fixed(140.0))
        .height(Length::Fill)
    };

    let left_panel = container(
        iced::widget::column![
            controls,
            output_view,
            footer
        ]
        .spacing(8),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    container(
        row![
            left_panel,
            right_panel
        ]
        .height(Length::Fill),
    )
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
