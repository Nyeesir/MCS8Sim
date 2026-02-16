#![windows_subsystem = "windows"]

use iced::daemon;

pub mod assembler;
#[cfg(test)] 
mod tests;
pub mod cpu;
pub mod gui;
pub mod encoding;

pub fn main() -> iced::Result {
    daemon(
        crate::gui::code_editor_app::CodeEditorApp::new,
        crate::gui::code_editor_app::CodeEditorApp::update,
        crate::gui::code_editor_app::CodeEditorApp::view,
    )
    .subscription(crate::gui::code_editor_app::CodeEditorApp::subscription)
    .run()
}
