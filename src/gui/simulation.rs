use iced::{window, Element, Length, Task};
use iced::widget::{button, container, row, scrollable, text};

const CHAR_WIDTH_PX: f32 = 8.0;
const CHAR_HEIGHT_PX: f32 = 16.0;
const CONSOLE_COLS: f32 = 80.0;
const CONSOLE_ROWS: f32 = 40.0;
const CONSOLE_PADDING: f32 = 16.0;

//TODO: ILOŚĆ INSTRUKCJI NA MINUTE, ZOBACZYĆ CZY NIE MAMY LIMITU

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
    start: Message,
    stop: Message,
    reset: Message,
) -> Element<'a, Message> {
    let controls = row![
        button("Start").on_press(start),
        button("Stop").on_press(stop),
        button("Reset").on_press(reset),
    ]
        .spacing(8);

    let content = text(output).font(iced::Font::MONOSPACE).size(14);
    let output_view = scrollable(container(content).padding(CONSOLE_PADDING))
        .width(Length::Fill)
        .height(Length::Fill);

    container(
        iced::widget::column![
            controls,
            output_view
        ]
            .spacing(8),
    )
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
