use iced::{alignment, widget::{
    button, checkbox, column, container, row, scrollable, text, text_editor,
    text_input,
}, Element, Length, Subscription, Task, Theme};
use iced::advanced::text::Wrapping;
use iced::widget::{operation, Id};
use iced::advanced::widget::operation::scrollable as scroll_op;
use iced::border;
use iced::time;
use iced::window;
use iced::Size;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};
use crate::assembler::{Assembler, INSTRUCTIONS, PSEUDO_INSTRUCTIONS, DATA_STATEMENTS};
use crate::cpu::{Cpu, controller::SimulatorController};
use crate::gui::simulation;
use std::ops::Range;

/*
TODO:
- RED ERROR LINE - FINE FOR NOW
- SAVE TO BIN
- LOAD BIN
 */

const MIN_FONT_SIZE: f32 = 8.0;
const MAX_FONT_SIZE: f32 = 64.0;
const EDITOR_SCROLL_ID: &str = "editor_scroll";
const EXTERNAL_HSCROLL_ID: &str = "external_hscroll";
const EDITOR_LINE_HEIGHT: f32 = 1.3;
const EDITOR_PADDING: f32 = 5.0;
const MEMORY_SIZE: usize = u16::MAX as usize + 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    Instruction,
    Pseudo,
    Data,
    Comment,
}

struct SyntaxHighlighter {
    current_line: usize,
}

impl iced::advanced::text::highlighter::Highlighter for SyntaxHighlighter {
    type Settings = ();
    type Highlight = TokenKind;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Self::Highlight)>;

    fn new(_settings: &Self::Settings) -> Self {
        Self { current_line: 0 }
    }

    fn update(&mut self, _new_settings: &Self::Settings) {}

    fn change_line(&mut self, line: usize) {
        self.current_line = line;
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        let _line_index = self.current_line;
        self.current_line = self.current_line.saturating_add(1);

        let mut highlights: Vec<(Range<usize>, TokenKind)> = Vec::new();
        let mut code = line;
        if let Some(comment_start) = line.find(';') {
            highlights.push((comment_start..line.len(), TokenKind::Comment));
            code = &line[..comment_start];
        }

        let mut it = code.char_indices().peekable();
        while let Some((start, ch)) = it.next() {
            if !ch.is_ascii_alphabetic() {
                continue;
            }

            let mut end = start + ch.len_utf8();
            while let Some(&(idx, next)) = it.peek() {
                if next.is_ascii_alphanumeric() {
                    it.next();
                    end = idx + next.len_utf8();
                } else {
                    break;
                }
            }

            if let Some(&(idx, ':')) = it.peek() {
                it.next();
                end = idx + 1;
                if let Some(&(idx2, ':')) = it.peek() {
                    it.next();
                    end = idx2 + 1;
                }
            }

            let token = &code[start..end];
            let kind = classify_token(token);
            if let Some(kind) = kind {
                highlights.push((start..end, kind));
            }
        }

        highlights.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}

fn classify_token(token: &str) -> Option<TokenKind> {
    if INSTRUCTIONS.iter().any(|kw| token.eq_ignore_ascii_case(kw)) {
        Some(TokenKind::Instruction)
    } else if PSEUDO_INSTRUCTIONS
        .iter()
        .any(|kw| token.eq_ignore_ascii_case(kw))
    {
        Some(TokenKind::Pseudo)
    } else if DATA_STATEMENTS
        .iter()
        .any(|kw| token.eq_ignore_ascii_case(kw))
    {
        Some(TokenKind::Data)
    } else {
        None
    }
}

fn build_gutter_text(line_count: usize) -> String {
    (1..=line_count)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn copy_trimmed_nonzero_slice(src: &[u8], dest: &mut [u8]) -> Result<(), String> {
    if src.len() > dest.len() {
        return Err(format!(
            "Źródło ({}) jest większe niż pamięć ({})",
            src.len(),
            dest.len()
        ));
    }

    let start = src.iter().position(|&b| b != 0);
    let end = src.iter().rposition(|&b| b != 0);
    let (Some(start), Some(end)) = (start, end) else {
        return Ok(());
    };

    dest[start..=end].copy_from_slice(&src[start..=end]);
    Ok(())
}

fn normalize_output_chunk(chunk: &str) -> String {
    chunk.replace('\t', "    ")
}

struct SimulationState {
    output: String,
    receiver: Receiver<String>,
    controller: SimulatorController,
}

pub struct CodeEditorApp {
    code: text_editor::Content,
    font_size: f32,
    font_size_input: String,
    last_line_count: usize,
    gutter_text: String,
    max_line_len: usize,
    line_lengths: Vec<usize>,
    hscroll_x: f32,
    at_bottom: bool,
    error_message: Option<String>,
    error_line: Option<usize>,
    load_bios: bool,
    main_window: window::Id,
    simulation_windows: HashMap<window::Id, SimulationState>,
}

#[derive(Debug, Clone)]
pub enum Message {
    CodeChanged(text_editor::Action),
    FontInc,
    FontDec,
    FontSizeInputChanged(String),
    FontSizeSubmitted,
    HorizontalScrollChanged(HScrollSource, f32),
    EditorScrolled(f32),
    ToggleBios(bool),
    LoadFile,
    LoadFilePicked(Option<PathBuf>),
    FileLoaded(Result<String, String>),
    Run,
    RunDebug,
    SimTick(Instant),
    SimStart(window::Id),
    SimStop(window::Id),
    SimReset(window::Id),
    SimStep(window::Id),
    WindowOpened(window::Id),
    CloseRequested(window::Id),
    WindowClosed(window::Id),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HScrollSource {
    External,
}

impl CodeEditorApp {
    pub fn new() -> (Self, Task<Message>) {
        let mut content = text_editor::Content::with_text("ORG 800h\n");
        let line_count = content.line_count().max(1);
        let gutter_text = build_gutter_text(line_count);
        let line_lengths: Vec<usize> = content
            .lines()
            .map(|line| line.text.chars().count())
            .collect();
        let max_line_len = line_lengths.iter().copied().max().unwrap_or(0);
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
                error_line: None,
                gutter_text,
                max_line_len,
                line_lengths,
                hscroll_x: 0.0,
                at_bottom: true,
                load_bios: true,
                main_window,
                simulation_windows: HashMap::new(),
            },
            open_task.map(Message::WindowOpened),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let mut task = Task::none();
        match message {
            Message::CodeChanged(action) => {
                if let text_editor::Action::Scroll { lines } = action {
                    let line_height = iced::advanced::text::LineHeight::Relative(EDITOR_LINE_HEIGHT);
                    let line_height_px = line_height.to_absolute(iced::Pixels(self.font_size)).0;
                    let delta = scroll_op::AbsoluteOffset {
                        x: 0.0,
                        y: (lines as f32) * line_height_px,
                    };
                    return iced::advanced::widget::operate(
                        scroll_op::scroll_by(Id::new(EDITOR_SCROLL_ID), delta),
                    );
                }

                let edit_action = match &action {
                    text_editor::Action::Edit(edit) => Some(edit.clone()),
                    _ => None,
                };
                let should_scroll = matches!(
                    action,
                    text_editor::Action::Edit(text_editor::Edit::Enter)
                );
                let prev_line = self.code.cursor().position.line;
                self.code.perform(action);

                let mut grew = false;
                if let Some(edit_action) = edit_action {
                    let line_count = self.code.line_count();
                    grew = line_count > self.last_line_count;
                    self.last_line_count = line_count;
                    if grew {
                        self.gutter_text = build_gutter_text(self.last_line_count.max(1));
                    }

                    let needs_full_rebuild = matches!(
                        edit_action,
                        text_editor::Edit::Enter | text_editor::Edit::Indent | text_editor::Edit::Unindent
                    ) || matches!(
                        edit_action,
                        text_editor::Edit::Paste(ref text) if text.contains('\n')
                    );

                    if needs_full_rebuild || line_count != self.line_lengths.len() {
                        self.rebuild_line_cache();
                    } else if let Some(line) = self.code.line(prev_line) {
                        let new_len = line.text.chars().count();
                        let prev_len = self.line_lengths[prev_line];
                        self.line_lengths[prev_line] = new_len;
                        if new_len > self.max_line_len {
                            self.max_line_len = new_len;
                        } else if prev_len == self.max_line_len && new_len < prev_len {
                            self.max_line_len =
                                self.line_lengths.iter().copied().max().unwrap_or(0);
                        }
                    }
                }

                if should_scroll {
                    if self.at_bottom {
                        task = iced::advanced::widget::operate(scroll_op::snap_to(
                            Id::new(EDITOR_SCROLL_ID),
                            scroll_op::RelativeOffset { x: None, y: Some(1.0) },
                        ));
                    } else {
                        let line_height = iced::advanced::text::LineHeight::Relative(EDITOR_LINE_HEIGHT);
                        let line_height_px = line_height.to_absolute(iced::Pixels(self.font_size)).0;
                        let delta = scroll_op::AbsoluteOffset { x: 0.0, y: line_height_px };
                        task = iced::advanced::widget::operate(
                            scroll_op::scroll_by(Id::new(EDITOR_SCROLL_ID), delta),
                        );
                    }
                } else if grew && self.at_bottom {
                    task = iced::advanced::widget::operate(scroll_op::snap_to(
                        Id::new(EDITOR_SCROLL_ID),
                        scroll_op::RelativeOffset { x: None, y: Some(1.0) },
                    ));
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
            }
            Message::FontSizeSubmitted => {
                if let Ok(parsed) = self.font_size_input.trim().parse::<f32>() {
                    if parsed >= MAX_FONT_SIZE {
                        self.font_size = MAX_FONT_SIZE;
                        self.font_size_input = format!("{:.0}", MAX_FONT_SIZE);
                    } else if parsed <= MIN_FONT_SIZE {
                        self.font_size = MIN_FONT_SIZE;
                        self.font_size_input = format!("{:.0}", MIN_FONT_SIZE);
                    } else {
                        self.font_size = parsed;
                        self.font_size_input = format!("{:.0}", parsed);
                    }
                }
            }
            Message::HorizontalScrollChanged(source, x) => {
                let x = if x.is_finite() { x.clamp(0.0, 1.0) } else { 0.0 };
                if (x - self.hscroll_x).abs() > f32::EPSILON {
                    self.hscroll_x = x;
                }

                if source == HScrollSource::External {
                    task = iced::advanced::widget::operate(scroll_op::snap_to(
                        Id::new(EDITOR_SCROLL_ID),
                        scroll_op::RelativeOffset { x: Some(self.hscroll_x), y: None },
                    ));
                }
            }
            Message::EditorScrolled(y) => {
                self.at_bottom = if y.is_finite() { y >= 0.99 } else { true };
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
                        let line_lengths: Vec<usize> = text
                            .lines()
                            .map(|line| line.chars().count())
                            .collect();
                        let max_line_len = line_lengths.iter().copied().max().unwrap_or(0);
                        self.code = text_editor::Content::with_text(&text);
                        self.last_line_count = self.code.line_count();
                        self.gutter_text = build_gutter_text(self.last_line_count.max(1));
                        self.error_message = None;
                        self.error_line = None;
                        self.max_line_len = max_line_len;
                        self.line_lengths = line_lengths;
                    }
                    Err(err) => {
                        self.error_message = Some(err);
                        self.error_line = None;
                    }
                }
            }
            Message::Run | Message::RunDebug => {
                let mut assembler = Assembler::new();
                match assembler.assemble(&self.code.text()) {
                    Ok(assembled) => {
                        let mut memory = [0u8; MEMORY_SIZE];

                        if self.load_bios {
                            let bios = match std::fs::read("src/bios.bin") {
                                Ok(bios) => bios,
                                Err(err) => {
                                    self.error_message =
                                        Some(format!("Nie można odczytać BIOS-u: {err}"));
                                    self.error_line = None;
                                    return Task::none();
                                }
                            };

                            if let Err(err) = copy_trimmed_nonzero_slice(&bios, &mut memory) {
                                self.error_message = Some(err);
                                self.error_line = None;
                                return Task::none();
                            }
                        }

                        if let Err(err) = copy_trimmed_nonzero_slice(&assembled, &mut memory) {
                            self.error_message = Some(err);
                            self.error_line = None;
                            return Task::none();
                        }

                        self.error_message = None;
                        self.error_line = None;
                        let (sim_window, open_task) = simulation::open_window();
                        let (tx, rx) = mpsc::channel();
                        let controller = SimulatorController::new(Cpu::with_memory(memory), Some(tx));
                        controller.run();
                        self.simulation_windows.insert(
                            sim_window,
                            SimulationState {
                                output: String::new(),
                                receiver: rx,
                                controller,
                            },
                        );
                        task = open_task.map(Message::WindowOpened);
                    }
                    Err(err) => {
                        self.error_line = err.line_number.checked_sub(1);
                        self.error_message = Some(err.to_string());
                    }
                }
            }
            Message::SimTick(_) => {
                for state in self.simulation_windows.values_mut() {
                    for chunk in state.receiver.try_iter() {
                        let normalized = normalize_output_chunk(&chunk);
                        state.output.push_str(&normalized);
                    }
                }
            }
            Message::SimStart(id) => {
                if let Some(state) = self.simulation_windows.get_mut(&id) {
                    state.controller.run();
                }
            }
            Message::SimStop(id) => {
                if let Some(state) = self.simulation_windows.get_mut(&id) {
                    state.controller.stop();
                }
            }
            Message::SimReset(id) => {
                if let Some(state) = self.simulation_windows.get_mut(&id) {
                    state.controller.reset();
                    state.output.clear();
                }
            }
            Message::SimStep(id) => {
                if let Some(state) = self.simulation_windows.get_mut(&id) {
                    state.controller.step();
                }
            }
            Message::WindowOpened(_id) => {}
            Message::CloseRequested(id) => {
                if id == self.main_window {
                    return self.close_all_and_exit();
                }
                if let Some(state) = self.simulation_windows.remove(&id) {
                    state.controller.stop();
                }
            }
            Message::WindowClosed(id) => {
                if id == self.main_window {
                    return self.close_all_and_exit();
                }
                if let Some(state) = self.simulation_windows.remove(&id) {
                    state.controller.stop();
                }
            }
        }
        task
    }

    pub fn view(&self, window: window::Id) -> Element<'_, Message> {
        if let Some(state) = self.simulation_windows.get(&window) {
            return simulation::view(
                &state.output,
                Message::SimStart(window),
                Message::SimStop(window),
                Message::SimReset(window),
                Message::SimStep(window),
            );
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
            time::every(Duration::from_millis(16)).map(Message::SimTick),
        ])
    }
}

impl CodeEditorApp {
    fn editor_view(&self) -> Element<'_, Message> {
        let line_count = self.last_line_count.max(1);

        let gutter = text(&self.gutter_text)
            .size(self.font_size)
            .align_x(alignment::Horizontal::Right)
            .width(Length::Fill);

        let line_height = iced::advanced::text::LineHeight::Relative(EDITOR_LINE_HEIGHT);
        let line_height_px = line_height.to_absolute(iced::Pixels(self.font_size)).0;

        let editor = text_editor(&self.code)
            .on_action(Message::CodeChanged)
            .font(iced::Font::MONOSPACE)
            .size(self.font_size)
            .line_height(line_height)
            .padding(EDITOR_PADDING)
            .wrapping(Wrapping::None)
            .height(Length::Fill)
            .style(|theme, status| {
                let mut style = iced::widget::text_editor::default(theme, status);
                let palette = theme.extended_palette();
                style.background = iced::Background::Color(iced::Color {
                    a: 0.0,
                    ..palette.background.base.color
                });
                style
            })
            .highlight_with::<SyntaxHighlighter>((), |highlight, theme: &Theme| {
                let palette = theme.extended_palette();
                let color = match highlight {
                    TokenKind::Instruction => palette.primary.strong.color,
                    TokenKind::Pseudo => palette.warning.strong.color,
                    TokenKind::Data => palette.success.strong.color,
                    TokenKind::Comment => iced::Color {
                        a: 0.7,
                        ..palette.background.weak.text
                    },
                };
                iced::advanced::text::highlighter::Format {
                    color: Some(color),
                    font: None,
                }
            });

        let max_line_len = self.max_line_len as f32;
        let approx_char_width = self.font_size * 0.6;
        let editor_width = (max_line_len * approx_char_width + self.font_size * 2.0)
            .max(300.0);

        let highlight_overlay = self.error_line
            .filter(|&line| line < line_count)
            .map(|line| {
                let offset = EDITOR_PADDING + (line as f32 * line_height_px);
                let bar = container(iced::widget::Space::new().height(Length::Fixed(line_height_px)))
                    .width(Length::Fill)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        let color = iced::Color { a: 0.35, ..palette.danger.weak.color };
                        container::Style::default()
                            .background(color)
                    });
                column![
                    iced::widget::Space::new().height(Length::Fixed(offset)),
                    bar,
                    iced::widget::Space::new().height(Length::Fill),
                ]
                .width(Length::Fill)
                .height(Length::Fill)
            })
            .unwrap_or_else(|| {
                column![iced::widget::Space::new().height(Length::Fill)]
                    .width(Length::Fill)
                    .height(Length::Fill)
            });

        let editor_stack = iced::widget::stack![
            highlight_overlay,
            editor
        ]
        .width(Length::Fixed(editor_width))
        .height(Length::Fill);

        let gutter_width = 38.0 * (self.font_size / 14.0);
        let editor_vscroll = scrollable(
            row![
                container(gutter)
                    .width(Length::Fixed(gutter_width))
                    .padding(5)
                    .align_x(alignment::Horizontal::Right),
                container(editor_stack)
                    .width(Length::Fixed(editor_width))
                    .height(Length::Fill)
            ]
        )
            .direction(scrollable::Direction::Both {
                vertical: scrollable::Scrollbar::default(),
                horizontal: scrollable::Scrollbar::hidden(),
            })
            .anchor_left()
            .id(Id::new(EDITOR_SCROLL_ID))
            .on_scroll(|viewport| Message::EditorScrolled(viewport.relative_offset().y))
            .width(Length::Fill)
            .height(Length::Fill);

        let external_hscroll = scrollable(
            container(
                iced::widget::Space::new()
                    .width(Length::Fixed(editor_width))
                    .height(Length::Fixed(1.0))
            )
        )
            .id(Id::new(EXTERNAL_HSCROLL_ID))
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::default(),
            ))
            .anchor_left()
            .on_scroll(|viewport| {
                Message::HorizontalScrollChanged(
                    HScrollSource::External,
                    viewport.relative_offset().x,
                )
            })
            .width(Length::Fill)
            .height(Length::Fixed(16.0));

        column![
            editor_vscroll,
            row![
                iced::widget::Space::new().width(Length::Fixed(gutter_width)),
                external_hscroll
            ]
            .height(Length::Fixed(16.0))
        ]
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
                    .on_submit(Message::FontSizeSubmitted)
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
            .simulation_windows
            .keys()
            .copied()
            .map(window::close::<Message>)
            .collect();
        tasks.push(iced::exit());
        Task::batch(tasks)
    }
}

impl CodeEditorApp {
    fn rebuild_line_cache(&mut self) {
        self.line_lengths = self
            .code
            .lines()
            .map(|line| line.text.chars().count())
            .collect();
        self.max_line_len = self.line_lengths.iter().copied().max().unwrap_or(0);
        self.last_line_count = self.line_lengths.len();
    }
}


