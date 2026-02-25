use iced::advanced::text::{Wrapping};
use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, text, text_editor, text_input,
};
use iced::{alignment, border, window, Element, Length, Theme};

use crate::gui::{deassembly, memory, registers, simulation};
use crate::gui::preferences::AppTheme;

use super::syntax::{SyntaxHighlighter, TokenKind};
use super::{
    CodeEditorApp, HScrollSource, Message, EDITOR_LINE_HEIGHT, EDITOR_PADDING, EDITOR_SCROLL_ID,
    EXTERNAL_HSCROLL_ID,
};

impl CodeEditorApp {
    pub fn view(&self, window: window::Id) -> Element<'_, Message> {
        if let Some(state) = self
            .simulation_windows
            .values()
            .find(|state| state.register_window_id == Some(window))
        {
            return registers::view(&state.register_state);
        }
        if let Some(state) = self
            .simulation_windows
            .values()
            .find(|state| state.deassembly_window_id == Some(window))
        {
            return deassembly::view(&state.deassembly_entries);
        }
        if let Some(state) = self
            .simulation_windows
            .values()
            .find(|state| state.memory_window_id == Some(window))
        {
            return memory::view(
                &state.memory_text,
                state.memory_start_row,
                move |viewport| Message::SimMemoryScrolled(window, viewport.absolute_offset().y),
            );
        }
        if let Some(state) = self.simulation_windows.get(&window) {
            return simulation::view(
                &state.output,
                state.waiting_for_input,
                state.debug_mode,
                state.cycles_per_second,
                state.is_halted,
                state.is_running,
                &state.cycles_limit_input,
                move |value| Message::SimCyclesLimitInputChanged(window, value),
                Message::SimCyclesLimitSubmitted(window),
                Message::SimToggleRegisters(window),
                Message::SimToggleDeassembly(window),
                Message::SimToggleMemory(window),
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
            row![editor, right_panel].height(Length::Fill),
            error_bar,
            bottom_bar
        ]
        .into()
    }

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
        let editor_width =
            (max_line_len * approx_char_width + self.font_size * 2.0).max(300.0);

        let highlight_overlay = self
            .error_line
            .filter(|&line| line < line_count)
            .map(|line| {
                let offset = EDITOR_PADDING + (line as f32 * line_height_px);
                let bar = container(
                    iced::widget::Space::new().height(Length::Fixed(line_height_px)),
                )
                .width(Length::Fill)
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    let color = iced::Color {
                        a: 0.35,
                        ..palette.danger.weak.color
                    };
                    container::Style::default().background(color)
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

        let editor_stack = iced::widget::stack![highlight_overlay, editor]
            .width(Length::Fixed(editor_width))
            .height(Length::Fill);

        let gutter_width = 38.0 * (self.font_size / 14.0);
        let editor_vscroll = scrollable(row![
            container(gutter)
                .width(Length::Fixed(gutter_width))
                .padding(5)
                .align_x(alignment::Horizontal::Right),
            container(editor_stack)
                .width(Length::Fixed(editor_width))
                .height(Length::Fill)
        ])
        .direction(scrollable::Direction::Both {
            vertical: scrollable::Scrollbar::default(),
            horizontal: scrollable::Scrollbar::hidden(),
        })
        .anchor_left()
        .id(iced::widget::Id::new(EDITOR_SCROLL_ID))
        .on_scroll(|viewport| Message::EditorScrolled(viewport.relative_offset().y))
        .width(Length::Fill)
        .height(Length::Fill);

        let external_hscroll = scrollable(container(
            iced::widget::Space::new()
                .width(Length::Fixed(editor_width))
                .height(Length::Fixed(1.0)),
        ))
        .id(iced::widget::Id::new(EXTERNAL_HSCROLL_ID))
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
                text("Theme").width(Length::Fixed(120.0)),
                pick_list(AppTheme::ALL, Some(self.theme), Message::ThemeSelected)
                    .width(Length::Fixed(200.0)),
                text(format!("Font: {:.0}", self.font_size)).width(Length::Fixed(120.0)),
                text_input("Size", &self.font_size_input)
                    .on_input(Message::FontSizeInputChanged)
                    .on_submit(Message::FontSizeSubmitted)
                    .width(Length::Fixed(120.0)),
                button("Font -")
                    .on_press(Message::FontDec)
                    .width(Length::Fixed(120.0)),
                button("Font +")
                    .on_press(Message::FontInc)
                    .width(Length::Fixed(120.0)),
            ]
            .spacing(8)
            .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fixed(200.0))
        .padding(8)
        .into()
    }

    fn error_bar(&self) -> Element<'_, Message> {
        if let Some(message) = &self.error_message {
            let dismiss = button(text("x").size(14))
                .padding([0, 6])
                .on_press(Message::CloseError);

            container(
                row![
                    text(message).size(14),
                    iced::widget::Space::new().width(Length::Fill),
                    dismiss
                ]
                .align_y(alignment::Vertical::Center),
            )
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
                button("Compile to bin").on_press(Message::CompileToBin),
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
