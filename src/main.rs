use iced::{application, Size};
use iced::window;

pub mod assembler;
#[cfg(test)] 
mod tests;
pub mod cpu;
pub mod gui;
pub mod deassembler;

pub fn main() -> iced::Result {
    application(
        crate::gui::code_editor_app::CodeEditorApp::new,
        crate::gui::code_editor_app::CodeEditorApp::update,
        crate::gui::code_editor_app::CodeEditorApp::view,
    )
    .window(window::Settings {
        size: Size::new(1024.0, 768.0),
        min_size: Some(Size::new(1024.0, 768.0)),
        ..window::Settings::default()
    })
    .run()
}
