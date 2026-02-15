mod controller;
mod syntax;
mod utils;
mod view;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::time::Instant;

use iced::widget::text_editor;
use iced::window;

use crate::cpu::{controller::SimulatorController, CpuState};

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

struct SimulationState {
    output: String,
    receiver: Receiver<String>,
    controller: SimulatorController,
    input_sender: mpsc::Sender<u8>,
    input_status_receiver: Receiver<bool>,
    waiting_for_input: bool,
    debug_mode: bool,
    cycles_receiver: Receiver<u64>,
    cycles_per_second: u64,
    halted_receiver: Receiver<bool>,
    is_halted: bool,
    is_running: bool,
    register_window_id: Option<window::Id>,
    register_receiver: Option<Receiver<CpuState>>,
    register_state: CpuState,
    cycles_limit_input: String,
    cycles_limit: Option<u64>,
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
    SimKeyInput(window::Id, u8),
    SimCyclesLimitInputChanged(window::Id, String),
    SimCyclesLimitSubmitted(window::Id),
    SimOpenRegisters(window::Id),
    WindowOpened(window::Id),
    CloseRequested(window::Id),
    WindowClosed(window::Id),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HScrollSource {
    External,
}
