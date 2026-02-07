use iced::{window, Element, Length, Task};
use iced::widget::{container, Space};

pub fn open_window() -> (window::Id, Task<window::Id>) {
    window::open(window::Settings::default())
}

pub fn view<'a, Message: 'a>() -> Element<'a, Message> {
    container(Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
