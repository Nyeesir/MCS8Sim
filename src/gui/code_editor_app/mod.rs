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

use crate::cpu::{io_handler::OutputEvent, simulation_controller::SimulationController, CpuState, InstructionTrace};
use crate::gui::preferences::{AppTheme, Preferences};

const MIN_FONT_SIZE: f32 = 8.0;
const MAX_FONT_SIZE: f32 = 64.0;
const EDITOR_SCROLL_ID: &str = "editor_scroll";
const EXTERNAL_HSCROLL_ID: &str = "external_hscroll";
const EDITOR_LINE_HEIGHT: f32 = 1.3;
const EDITOR_PADDING: f32 = 5.0;
const MEMORY_SIZE: usize = u16::MAX as usize + 1;

struct SimulationState {
    output: String,
    receiver: Receiver<OutputEvent>,
    controller: SimulationController,
    input_sender: mpsc::Sender<u8>,
    input_status_receiver: Receiver<bool>,
    waiting_for_input: bool,
    input_pending: bool,
    is_focused: bool,
    debug_mode: bool,
    cycles_receiver: Receiver<u64>,
    cycles_per_second: u64,
    halted_receiver: Receiver<bool>,
    is_halted: bool,
    is_running: bool,
    register_window_id: Option<window::Id>,
    register_receiver: Option<Receiver<CpuState>>,
    register_state: CpuState,
    deassembly_window_id: Option<window::Id>,
    deassembly_receiver: Option<Receiver<InstructionTrace>>,
    deassembly_entries: Vec<InstructionTrace>,
    memory_window_id: Option<window::Id>,
    memory_receiver: Option<Receiver<Vec<u8>>>,
    memory_snapshot: Vec<u8>,
    memory_text: String,
    memory_start_row: usize,
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
    theme: AppTheme,
    main_window: window::Id,
    simulation_windows: HashMap<window::Id, SimulationState>,
    preferences: Preferences,
    window_kinds: HashMap<window::Id, WindowKind>,
}

impl CodeEditorApp {
    pub fn theme(&self) -> iced::Theme {
        match self.theme {
            AppTheme::Light => iced::Theme::Light,
            AppTheme::Dark => iced::Theme::Dark,
            AppTheme::Dracula => iced::Theme::Dracula,
            AppTheme::Nord => iced::Theme::Nord,
            AppTheme::SolarizedLight => iced::Theme::SolarizedLight,
            AppTheme::SolarizedDark => iced::Theme::SolarizedDark,
            AppTheme::GruvboxLight => iced::Theme::GruvboxLight,
            AppTheme::GruvboxDark => iced::Theme::GruvboxDark,
            AppTheme::CatppuccinLatte => iced::Theme::CatppuccinLatte,
            AppTheme::CatppuccinFrappe => iced::Theme::CatppuccinFrappe,
            AppTheme::CatppuccinMacchiato => iced::Theme::CatppuccinMacchiato,
            AppTheme::CatppuccinMocha => iced::Theme::CatppuccinMocha,
            AppTheme::TokyoNight => iced::Theme::TokyoNight,
            AppTheme::TokyoNightStorm => iced::Theme::TokyoNightStorm,
            AppTheme::TokyoNightLight => iced::Theme::TokyoNightLight,
            AppTheme::KanagawaWave => iced::Theme::KanagawaWave,
            AppTheme::KanagawaDragon => iced::Theme::KanagawaDragon,
            AppTheme::KanagawaLotus => iced::Theme::KanagawaLotus,
            AppTheme::Moonfly => iced::Theme::Moonfly,
            AppTheme::Nightfly => iced::Theme::Nightfly,
            AppTheme::Oxocarbon => iced::Theme::Oxocarbon,
            AppTheme::Ferra => iced::Theme::Ferra,
        }
    }
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
    ThemeSelected(AppTheme),
    LoadFile,
    LoadFilePicked(Option<PathBuf>),
    FileLoaded(Result<String, String>),
    CloseError,
    Run,
    RunDebug,
    CompileToBin,
    CompileToBinPicked(Option<PathBuf>, Vec<u8>),
    CompileToBinSaved(Result<(), String>),
    SimTick(Instant),
    SimStart(window::Id),
    SimStop(window::Id),
    SimReset(window::Id),
    SimStep(window::Id),
    SimKeyInput(window::Id, u8),
    SimCyclesLimitInputChanged(window::Id, String),
    SimCyclesLimitSubmitted(window::Id),
    SimToggleRegisters(window::Id),
    SimToggleDeassembly(window::Id),
    SimToggleMemory(window::Id),
    SimMemoryScrolled(window::Id, f32),
    WindowEvent(window::Id, window::Event),
    WindowOpened(window::Id),
    CloseRequested(window::Id),
    WindowClosed(window::Id),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HScrollSource {
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum WindowKind {
    Main,
    Simulation,
    SimulationDebug,
    Registers,
    Deassembly,
    Memory,
}
