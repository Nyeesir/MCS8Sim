use iced::{alignment, widget::{
    button, checkbox, column, container, row, scrollable, text, text_editor,
    text_input,
}, Element, Length, Subscription, Task, Theme};
use iced::advanced::text::Wrapping;
use iced::widget::{operation, Id};
use iced::border;
use iced::window;
use iced::Size;
use std::collections::HashSet;
use std::path::PathBuf;
use crate::assembler::Assembler;
use crate::gui::simulation;

const MIN_FONT_SIZE: f32 = 8.0;
const MAX_FONT_SIZE: f32 = 64.0;
const EDITOR_SCROLL_ID: &str = "editor_scroll";

/*
TODO:
- LOAD BIOS
- JUMP TO ERROR
- HIGHLIGHTING
- SAVE TO BIN
- LOAD BIN

- CHECKING START AND END OF CODE, INSERTING INTO RIGHT PLACE
 */

pub struct CodeEditorApp {
    code: text_editor::Content,
    font_size: f32,
    font_size_input: String,
    last_line_count: usize,
    error_message: Option<String>,
    load_bios: bool,
    main_window: window::Id,
    blank_windows: HashSet<window::Id>,
}

#[derive(Debug, Clone)]
pub enum Message {
    CodeChanged(text_editor::Action),
    FontInc,
    FontDec,
    FontSizeInputChanged(String),
    ToggleBios(bool),
    LoadFile,
    LoadFilePicked(Option<PathBuf>),
    FileLoaded(Result<String, String>),
    Run,
    RunDebug,
    WindowOpened(window::Id),
    CloseRequested(window::Id),
    WindowClosed(window::Id),
}

impl CodeEditorApp {
    pub fn new() -> (Self, Task<Message>) {
        let mut content = text_editor::Content::with_text("ORG 800h\n");
        let (main_window, open_task) = window::open(window::Settings {
            size: Size::new(1024.0, 768.0),
            min_size: Some(Size::new(1024.0, 768.0)),
            ..window::Settings::default()
        });
        (
            Self {
                last_line_count: content.line_count(),
                code: content,
                font_size: 14.0,
                font_size_input: "14".to_string(),
                error_message: None,
                load_bios: true,
                main_window,
                blank_windows: HashSet::new(),
            },
            open_task.map(Message::WindowOpened),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let mut task = Task::none();
        match message {
            Message::CodeChanged(action) => {
                let should_scroll = matches!(
                    action,
                    text_editor::Action::Edit(text_editor::Edit::Enter)
                );
                self.code.perform(action);

                let line_count = self.code.line_count();
                let grew = line_count > self.last_line_count;
                self.last_line_count = line_count;

                if should_scroll || grew {
                    task = operation::snap_to_end(Id::new(EDITOR_SCROLL_ID));
                }
            }
            Message::FontInc => {
                self.font_size = (self.font_size + 1.0).clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
                self.font_size_input = format!("{:.0}", self.font_size);
            }
            Message::FontDec => {
                self.font_size = (self.font_size - 1.0).clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
                self.font_size_input = format!("{:.0}", self.font_size);
            }
            Message::FontSizeInputChanged(value) => {
                self.font_size_input = value;
                if let Ok(parsed) = self.font_size_input.trim().parse::<f32>() {
                    if parsed >= MAX_FONT_SIZE {
                        self.font_size = MAX_FONT_SIZE;
                        self.font_size_input = format!("{:.0}", MAX_FONT_SIZE);
                    }
                    else if parsed <= MIN_FONT_SIZE {
                        self.font_size = MIN_FONT_SIZE;
                    } else {
                        self.font_size = parsed;
                    }
                }
            }
            Message::ToggleBios(v) => self.load_bios = v,
            Message::LoadFile => {
                task = Task::perform(
                    async {
                        rfd::FileDialog::new()
                            .add_filter("Text", &["txt","asm"])
                            .pick_file()
                    },
                    Message::LoadFilePicked,
                );
            }
            Message::LoadFilePicked(path) => {
                if let Some(path) = path {
                    task = Task::perform(
                        async move {
                            std::fs::read_to_string(&path)
                                .map_err(|e| format!("Nie można odczytać pliku: {e}"))
                        },
                        Message::FileLoaded,
                    );
                }
            }
            Message::FileLoaded(result) => {
                match result {
                    Ok(text) => {
                        self.code = text_editor::Content::with_text(&text);
                        self.last_line_count = self.code.line_count();
                        self.error_message = None;
                    }
                    Err(err) => {
                        self.error_message = Some(err);
                    }
                }
            }
            Message::Run | Message::RunDebug => {
                let mut assembler = Assembler::new();
                match assembler.assemble(&self.code.text()) {
                    Ok(_) => {
                        self.error_message = None;
                        let (blank_window, open_task) = simulation::open_window();
                        self.blank_windows.insert(blank_window);
                        task = open_task.map(Message::WindowOpened);
                    }
                    Err(err) => {
                        self.error_message = Some(err.to_string());
                    }
                }
            }
            Message::WindowOpened(_) => {}
            Message::CloseRequested(id) => {
                if id == self.main_window {
                    return self.close_all_and_exit();
                }
                self.blank_windows.remove(&id);
            }
            Message::WindowClosed(id) => {
                if id == self.main_window {
                    return self.close_all_and_exit();
                }
                self.blank_windows.remove(&id);
            }
        }
        task
    }

    pub fn view(&self, window: window::Id) -> Element<'_, Message> {
        if self.blank_windows.contains(&window) {
            return simulation::view();
        }

        let editor = self.editor_view();
        let right_panel = self.right_panel();
        let bottom_bar = self.bottom_bar();
        let error_bar = self.error_bar();

        column![
            row![
                editor,
                right_panel
            ]
            .height(Length::Fill),
            error_bar,
            bottom_bar
        ].into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            window::close_requests().map(Message::CloseRequested),
            window::close_events().map(Message::WindowClosed),
        ])
    }
}

impl CodeEditorApp {
    fn editor_view(&self) -> Element<'_, Message> {
        let code_text = self.code.text();
        let line_count = code_text.lines().count().max(1);

        let gutter = text(
            (1..=line_count)
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        )
            .size(self.font_size)
            .align_x(alignment::Horizontal::Right)
            .width(Length::Fill);

        let editor = text_editor(&self.code)
            .on_action(Message::CodeChanged)
            .font(iced::Font::MONOSPACE)
            .size(self.font_size)
            .wrapping(Wrapping::None)
            .height(Length::Fill);

        let max_line_len = code_text
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0) as f32;
        let approx_char_width = self.font_size * 0.6;
        let editor_width = (max_line_len * approx_char_width + self.font_size * 2.0)
            .max(300.0);

        let editor_hscroll = scrollable(
            container(editor).width(Length::Fixed(editor_width))
        )
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::default(),
            ))
            .width(Length::Fill)
            .height(Length::Fill);

        scrollable(
            row![
                container(gutter)
                    .width(Length::Fixed(38.0 * (self.font_size / 14.0)))
                    .padding(5)
                    .align_x(alignment::Horizontal::Right),
                editor_hscroll
            ]
        )
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::default(),
            ))
            .id(Id::new(EDITOR_SCROLL_ID))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn right_panel(&self) -> Element<'_, Message> {
        container(
            column![
                text(format!("Font: {:.0}", self.font_size))
                    .width(Length::Fixed(80.0)),
                text_input("Size", &self.font_size_input)
                    .on_input(Message::FontSizeInputChanged)
                    .width(Length::Fixed(80.0)),
                button("Font -").on_press(Message::FontDec)
                    .width(Length::Fixed(80.0)),
                button("Font +").on_press(Message::FontInc)
                    .width(Length::Fixed(80.0)),
            ]
                .spacing(8)
                .align_x(alignment::Horizontal::Center),
        )
            .width(Length::Fixed(120.0))
            .padding(8)
            .into()
    }

    fn error_bar(&self) -> Element<'_, Message> {
        if let Some(message) = &self.error_message {
            container(text(message).size(14))
                .padding(6)
                .width(Length::Fill)
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    container::Style::default()
                        .background(palette.danger.weak.color)
                        .border(border::rounded(3).color(palette.danger.strong.color))
                })
                .into()
        } else {
            container(iced::widget::Space::new().height(Length::Fixed(0.0))).into()
        }
    }

    fn bottom_bar(&self) -> Element<'_, Message> {
        container(
            row![
                button("Load file").on_press(Message::LoadFile),
                button("Run simulation").on_press(Message::Run),
                button("Run simulation with debug").on_press(Message::RunDebug),
                iced::widget::Space::new().width(Length::Fill),
                checkbox(self.load_bios)
                    .label("Load BIOS")
                    .on_toggle(Message::ToggleBios),
            ]
                .spacing(10)
                .align_y(alignment::Vertical::Center),
        )
            .padding(8)
            .into()
    }
}

impl CodeEditorApp {
    fn close_all_and_exit(&self) -> Task<Message> {
        let mut tasks: Vec<Task<Message>> = self
            .blank_windows
            .iter()
            .copied()
            .map(window::close::<Message>)
            .collect();
        tasks.push(iced::exit());
        Task::batch(tasks)
    }
}


