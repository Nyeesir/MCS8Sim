use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;

use iced::advanced::text::LineHeight;
use iced::advanced::widget::operation::scrollable as scroll_op;
use iced::widget::{text_editor, Id};
use iced::{event, keyboard, time, window, Point, Size, Subscription, Task};
use iced::keyboard::key::Named::Enter;

use crate::assembler::Assembler;
use crate::cpu::{simulation_controller::SimulationController, Cpu, CpuState, InstructionTrace};
use crate::encoding;
use crate::gui::{deassembly, preferences::Preferences, registers, simulation};

use super::utils::{build_gutter_text, copy_trimmed_nonzero_slice, normalize_output_chunk};
use super::{
    CodeEditorApp, HScrollSource, Message, SimulationState, WindowKind, EDITOR_LINE_HEIGHT,
    EDITOR_SCROLL_ID, MAX_FONT_SIZE, MEMORY_SIZE, MIN_FONT_SIZE,
};

impl CodeEditorApp {
    pub fn new() -> (Self, Task<Message>) {
        let mut preferences = Preferences::load();
        if !preferences.font_size.is_finite()
            || preferences.font_size < MIN_FONT_SIZE
            || preferences.font_size > MAX_FONT_SIZE
        {
            preferences.font_size = 14.0;
        }
        let mut content = text_editor::Content::with_text("ORG 800h\n");
        let line_count = content.line_count().max(1);
        let gutter_text = build_gutter_text(line_count);
        let line_lengths: Vec<usize> = content
            .lines()
            .map(|line| line.text.chars().count())
            .collect();
        let max_line_len = line_lengths.iter().copied().max().unwrap_or(0);
        let mut main_settings = window::Settings {
            size: Size::new(1024.0, 768.0),
            min_size: Some(Size::new(1024.0, 768.0)),
            ..window::Settings::default()
        };
        if let Some(mut geom) = preferences.main_window {
            if let Some(min) = main_settings.min_size {
                if geom.width < min.width {
                    geom.width = min.width;
                }
                if geom.height < min.height {
                    geom.height = min.height;
                }
            }
            geom.apply_to_settings(&mut main_settings);
        }
        let (main_window, open_task) = window::open(main_settings);
        let mut window_kinds = HashMap::new();
        window_kinds.insert(main_window, WindowKind::Main);
        (
            Self {
                last_line_count: content.line_count(),
                code: content,
                font_size: preferences.font_size,
                font_size_input: format!("{:.0}", preferences.font_size),
                error_message: None,
                error_line: None,
                gutter_text,
                max_line_len,
                line_lengths,
                hscroll_x: 0.0,
                at_bottom: true,
                load_bios: preferences.load_bios,
                main_window,
                simulation_windows: std::collections::HashMap::new(),
                preferences,
                window_kinds,
            },
            open_task.map(Message::WindowOpened),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let mut task = Task::none();
        match message {
            Message::CodeChanged(action) => {
                if let text_editor::Action::Scroll { lines } = action {
                    let line_height = LineHeight::Relative(EDITOR_LINE_HEIGHT);
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
                        text_editor::Edit::Enter
                            | text_editor::Edit::Indent
                            | text_editor::Edit::Unindent
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
                        let line_height = LineHeight::Relative(EDITOR_LINE_HEIGHT);
                        let line_height_px =
                            line_height.to_absolute(iced::Pixels(self.font_size)).0;
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
                self.preferences.font_size = self.font_size;
                self.preferences.save();
            }
            Message::FontDec => {
                self.font_size = (self.font_size - 1.0).clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
                self.font_size_input = format!("{:.0}", self.font_size);
                self.preferences.font_size = self.font_size;
                self.preferences.save();
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
                    self.preferences.font_size = self.font_size;
                    self.preferences.save();
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
            Message::ToggleBios(v) => {
                self.load_bios = v;
                self.preferences.load_bios = v;
                self.preferences.save();
            }
            Message::LoadFile => {
                task = Task::perform(
                    async {
                        rfd::FileDialog::new()
                            .add_filter("Text", &["txt", "asm"])
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
            Message::FileLoaded(result) => match result {
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
            },
            Message::Run | Message::RunDebug => {
                let debug_mode = matches!(message, Message::RunDebug);
                let mut assembler = Assembler::new();
                match assembler.assemble(&self.code.text()) {
                    Ok(assembled) => {
                        let mut memory = [0u8; MEMORY_SIZE];

                        if self.load_bios {
                            let bios_path = std::env::current_exe()
                                .ok()
                                .and_then(|path| path.parent().map(|dir| dir.join("bios.bin")))
                                .unwrap_or_else(|| "bios.bin".into());
                            let bios = match std::fs::read(&bios_path) {
                                Ok(bios) => bios,
                                Err(err) => {
                                    self.error_message =
                                        Some(format!("Can't read BIOS file ({:?}): {err}", bios_path));
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
                        let mut close_tasks: Vec<Task<Message>> = self
                            .simulation_windows
                            .keys()
                            .copied()
                            .map(window::close::<Message>)
                            .collect();
                        for state in self.simulation_windows.values() {
                            if let Some(reg_id) = state.register_window_id {
                                close_tasks.push(window::close::<Message>(reg_id));
                            }
                            if let Some(deasm_id) = state.deassembly_window_id {
                                close_tasks.push(window::close::<Message>(deasm_id));
                            }
                            state.controller.stop();
                        }
                        self.simulation_windows.clear();
                        self.window_kinds.retain(|_, kind| *kind == WindowKind::Main);

                        let sim_geometry = if debug_mode {
                            self.preferences.sim_debug_window
                        } else {
                            self.preferences.sim_window
                        };
                        let (sim_window, open_task) =
                            simulation::open_window_with_geometry(sim_geometry);
                        let (tx, rx) = mpsc::channel();
                        let (input_tx, input_rx) = mpsc::channel();
                        let (input_status_tx, input_status_rx) = mpsc::channel();
                        let (cycles_tx, cycles_rx) = mpsc::channel();
                        let (halted_tx, halted_rx) = mpsc::channel();
                        let (state_tx, state_rx) = mpsc::channel();
                        let (reg_window, reg_task, reg_receiver, state_sender) = if debug_mode
                            && self.preferences.show_registers
                        {
                            let (reg_id, task) =
                                registers::open_window_with_geometry(self.preferences.registers_window);
                            (Some(reg_id), Some(task), Some(state_rx), Some(state_tx))
                        } else {
                            (None, None, Some(state_rx), Some(state_tx))
                        };
                        let (deasm_window, deasm_task) = if debug_mode
                            && self.preferences.show_deassembly
                        {
                            let (deasm_id, task) =
                                deassembly::open_window_with_geometry(self.preferences.deassembly_window);
                            (Some(deasm_id), Some(task))
                        } else {
                            (None, None)
                        };
                        let (trace_sender, trace_receiver) = if debug_mode {
                            let (tx, rx) = mpsc::channel::<InstructionTrace>();
                            (Some(tx), Some(rx))
                        } else {
                            (None, None)
                        };
                        let controller = SimulationController::new(
                            Cpu::with_memory(memory),
                            Some(tx),
                            Some(input_rx),
                            Some(input_status_tx),
                            Some(cycles_tx),
                            Some(halted_tx),
                            state_sender,
                            trace_sender,
                            debug_mode.then_some(1000),
                        );
                        if !debug_mode {
                            controller.run();
                        }
                        self.simulation_windows.insert(
                            sim_window,
                            SimulationState {
                                output: String::new(),
                                receiver: rx,
                                controller,
                                input_sender: input_tx,
                                input_status_receiver: input_status_rx,
                                waiting_for_input: false,
                                debug_mode,
                                cycles_receiver: cycles_rx,
                                cycles_per_second: 0,
                                halted_receiver: halted_rx,
                                is_halted: false,
                                is_running: !debug_mode,
                                register_window_id: reg_window,
                                register_receiver: reg_receiver,
                                register_state: CpuState::default(),
                                deassembly_window_id: deasm_window,
                                deassembly_receiver: trace_receiver,
                                deassembly_entries: Vec::new(),
                                cycles_limit_input: debug_mode
                                    .then_some("1000".to_string())
                                    .unwrap_or_default(),
                                cycles_limit: debug_mode.then_some(1000),
                            },
                        );
                        self.window_kinds.insert(
                            sim_window,
                            if debug_mode {
                                WindowKind::SimulationDebug
                            } else {
                                WindowKind::Simulation
                            },
                        );
                        if let Some(reg_id) = reg_window {
                            self.window_kinds.insert(reg_id, WindowKind::Registers);
                        }
                        if let Some(deasm_id) = deasm_window {
                            self.window_kinds.insert(deasm_id, WindowKind::Deassembly);
                        }
                        let mut tasks = close_tasks;
                        tasks.push(open_task.map(Message::WindowOpened));
                        if let Some(reg_task) = reg_task {
                            tasks.push(reg_task.map(Message::WindowOpened));
                        }
                        if let Some(deasm_task) = deasm_task {
                            tasks.push(deasm_task.map(Message::WindowOpened));
                        }
                        task = Task::batch(tasks);
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
                    for status in state.input_status_receiver.try_iter() {
                        state.waiting_for_input = status;
                    }
                    for cycles in state.cycles_receiver.try_iter() {
                        state.cycles_per_second = cycles;
                    }
                    for halted in state.halted_receiver.try_iter() {
                        state.is_halted = halted;
                        if halted {
                            state.is_running = false;
                        }
                    }
                    if let Some(rx) = state.register_receiver.as_ref() {
                        for snapshot in rx.try_iter() {
                            state.register_state = snapshot;
                        }
                    }
                    if let Some(rx) = state.deassembly_receiver.as_ref() {
                        for trace in rx.try_iter() {
                            state.deassembly_entries.push(trace);
                            if state.deassembly_entries.len() > 11 {
                                let excess = state.deassembly_entries.len() - 11;
                                state.deassembly_entries.drain(0..excess);
                            }
                        }
                    }
                }
            }
            Message::SimStart(id) => {
                if let Some(state) = self.simulation_windows.get_mut(&id) {
                    state.controller.run();
                    state.is_running = true;
                }
            }
            Message::SimStop(id) => {
                if let Some(state) = self.simulation_windows.get_mut(&id) {
                    state.controller.stop();
                    state.is_running = false;
                }
            }
            Message::SimReset(id) => {
                if let Some(state) = self.simulation_windows.get_mut(&id) {
                    state.controller.reset();
                    state.output.clear();
                    state.is_running = false;
                    state.deassembly_entries.clear();
                }
            }
            Message::SimStep(id) => {
                if let Some(state) = self.simulation_windows.get_mut(&id) {
                    state.controller.step();
                }
            }
            Message::SimCyclesLimitInputChanged(id, value) => {
                if let Some(state) = self.simulation_windows.get_mut(&id) {
                    state.cycles_limit_input = value;
                }
            }
            Message::SimCyclesLimitSubmitted(id) => {
                if let Some(state) = self.simulation_windows.get_mut(&id) {
                    let trimmed = state.cycles_limit_input.trim();
                    if trimmed.is_empty() {
                        state.cycles_limit = None;
                        state.controller.set_cycles_limit(None);
                    } else if let Ok(limit) = trimmed.parse::<u64>() {
                        let clamped = if limit == 0 {
                            6_000_000
                        } else {
                            limit.min(6_000_000)
                        };
                        state.cycles_limit = Some(clamped);
                        state.cycles_limit_input = clamped.to_string();
                        state.controller.set_cycles_limit(Some(clamped));
                    }
                }
            }
            Message::SimToggleRegisters(id) => {
                if let Some(state) = self.simulation_windows.get_mut(&id) {
                    if !state.debug_mode {
                        return Task::none();
                    }
                    if let Some(reg_id) = state.register_window_id.take() {
                        self.preferences.show_registers = false;
                        self.preferences.save();
                        self.window_kinds.remove(&reg_id);
                        task = window::close::<Message>(reg_id);
                    } else {
                        let (reg_id, open_task) = registers::open_window_with_geometry(
                            self.preferences.registers_window,
                        );
                        state.register_window_id = Some(reg_id);
                        self.window_kinds.insert(reg_id, WindowKind::Registers);
                        self.preferences.show_registers = true;
                        self.preferences.save();
                        task = open_task.map(Message::WindowOpened);
                    }
                }
            }
            Message::SimToggleDeassembly(id) => {
                if let Some(state) = self.simulation_windows.get_mut(&id) {
                    if !state.debug_mode {
                        return Task::none();
                    }
                    if let Some(deasm_id) = state.deassembly_window_id.take() {
                        self.preferences.show_deassembly = false;
                        self.preferences.save();
                        self.window_kinds.remove(&deasm_id);
                        task = window::close::<Message>(deasm_id);
                    } else {
                        let (deasm_id, open_task) = deassembly::open_window_with_geometry(
                            self.preferences.deassembly_window,
                        );
                        state.deassembly_window_id = Some(deasm_id);
                        self.window_kinds.insert(deasm_id, WindowKind::Deassembly);
                        self.preferences.show_deassembly = true;
                        self.preferences.save();
                        task = open_task.map(Message::WindowOpened);
                    }
                }
            }
            Message::WindowEvent(id, window_event) => {
                match window_event {
                    window::Event::Moved(position) => {
                        self.update_window_geometry(id, Some(position), None);
                    }
                    window::Event::Resized(size) => {
                        self.update_window_geometry(id, None, Some(size));
                    }
                    window::Event::Opened { position, size } => {
                        self.update_window_geometry(id, position, Some(size));
                    }
                    _ => {}
                }
            }
            Message::SimKeyInput(id, value) => {
                if id == self.main_window {
                    return Task::none();
                }
                if let Some(state) = self.simulation_windows.get_mut(&id) {
                    let _ = state.input_sender.send(value);
                }
            }
            Message::WindowOpened(_id) => {}
            Message::CloseRequested(id) => {
                if id == self.main_window {
                    return self.close_all_and_exit();
                }
                if let Some(state) = self.simulation_windows.remove(&id) {
                    let mut tasks: Vec<Task<Message>> = Vec::new();
                    if let Some(reg_id) = state.register_window_id {
                        tasks.push(window::close::<Message>(reg_id));
                        self.window_kinds.remove(&reg_id);
                    }
                    if let Some(deasm_id) = state.deassembly_window_id {
                        tasks.push(window::close::<Message>(deasm_id));
                        self.window_kinds.remove(&deasm_id);
                    }
                    self.window_kinds.remove(&id);
                    state.controller.stop();
                    if !tasks.is_empty() {
                        task = Task::batch(tasks);
                    }
                } else {
                    for state in self.simulation_windows.values_mut() {
                        if state.register_window_id == Some(id) {
                            state.register_window_id = None;
                            self.preferences.show_registers = false;
                            self.preferences.save();
                            self.window_kinds.remove(&id);
                        }
                        if state.deassembly_window_id == Some(id) {
                            state.deassembly_window_id = None;
                            self.preferences.show_deassembly = false;
                            self.preferences.save();
                            self.window_kinds.remove(&id);
                        }
                    }
                }
            }
            Message::WindowClosed(id) => {
                if id == self.main_window {
                    return self.close_all_and_exit();
                }
                if let Some(state) = self.simulation_windows.remove(&id) {
                    let mut tasks: Vec<Task<Message>> = Vec::new();
                    if let Some(reg_id) = state.register_window_id {
                        tasks.push(window::close::<Message>(reg_id));
                        self.window_kinds.remove(&reg_id);
                    }
                    if let Some(deasm_id) = state.deassembly_window_id {
                        tasks.push(window::close::<Message>(deasm_id));
                        self.window_kinds.remove(&deasm_id);
                    }
                    self.window_kinds.remove(&id);
                    state.controller.stop();
                    if !tasks.is_empty() {
                        task = Task::batch(tasks);
                    }
                } else {
                    for state in self.simulation_windows.values_mut() {
                        if state.register_window_id == Some(id) {
                            state.register_window_id = None;
                            self.preferences.show_registers = false;
                            self.preferences.save();
                            self.window_kinds.remove(&id);
                        }
                        if state.deassembly_window_id == Some(id) {
                            state.deassembly_window_id = None;
                            self.preferences.show_deassembly = false;
                            self.preferences.save();
                            self.window_kinds.remove(&id);
                        }
                    }
                }
            }
        }
        task
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            window::close_requests().map(Message::CloseRequested),
            window::close_events().map(Message::WindowClosed),
            time::every(Duration::from_millis(16)).map(Message::SimTick),
            event::listen_with(|event, status, id| {
                if status == event::Status::Captured {
                    return None;
                }
                match event {
                    iced::Event::Keyboard(key_event) => match key_event {
                        keyboard::Event::KeyPressed { key, text, .. } => {
                            if key == keyboard::Key::Named(Enter) {
                                return Some(Message::SimKeyInput(id, 0x0D));
                            }
                            text.and_then(|text| text.chars().next())
                                .and_then(|ch| encoding::cp1252_encode(ch))
                                .map(|value| Message::SimKeyInput(id, value))
                        }
                        _ => None,
                    },
                    iced::Event::Window(window_event) => {
                        Some(Message::WindowEvent(id, window_event))
                    }
                    _ => None,
                }
            }),
        ])
    }

    fn close_all_and_exit(&self) -> Task<Message> {
        let mut tasks: Vec<Task<Message>> = self
            .simulation_windows
            .keys()
            .copied()
            .map(window::close::<Message>)
            .collect();
        for state in self.simulation_windows.values() {
            if let Some(reg_id) = state.register_window_id {
                tasks.push(window::close::<Message>(reg_id));
            }
            if let Some(deasm_id) = state.deassembly_window_id {
                tasks.push(window::close::<Message>(deasm_id));
            }
        }
        tasks.push(iced::exit());
        Task::batch(tasks)
    }

    fn rebuild_line_cache(&mut self) {
        self.line_lengths = self
            .code
            .lines()
            .map(|line| line.text.chars().count())
            .collect();
        self.max_line_len = self.line_lengths.iter().copied().max().unwrap_or(0);
        self.last_line_count = self.line_lengths.len();
    }

    fn update_window_geometry(
        &mut self,
        id: window::Id,
        position: Option<Point>,
        size: Option<Size>,
    ) {
        let Some(kind) = self.window_kinds.get(&id).copied() else {
            return;
        };

        let target = match kind {
            WindowKind::Main => &mut self.preferences.main_window,
            WindowKind::Simulation => &mut self.preferences.sim_window,
            WindowKind::SimulationDebug => &mut self.preferences.sim_debug_window,
            WindowKind::Registers => &mut self.preferences.registers_window,
            WindowKind::Deassembly => &mut self.preferences.deassembly_window,
        };

        let mut geom = target.unwrap_or_default();
        if let Some(pos) = position {
            if pos.x.is_finite()
                && pos.y.is_finite()
                && pos.x > -10000.0
                && pos.y > -10000.0
            {
                geom.x = pos.x;
                geom.y = pos.y;
            }
        }
        if let Some(size) = size {
            if size.width >= 50.0 && size.height >= 50.0 {
                geom.width = size.width;
                geom.height = size.height;
            }
        }
        *target = Some(geom);
        self.preferences.save();
    }
}
