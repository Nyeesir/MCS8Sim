use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use iced::{window, Point, Size};

const PREFERENCES_FILE: &str = "mcs8sim_prefs.toml";

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl WindowGeometry {
    pub fn apply_to_settings(&self, settings: &mut window::Settings) {
        if self.width > 0.0 && self.height > 0.0 {
            settings.size = Size::new(self.width, self.height);
        }
        if self.x.is_finite()
            && self.y.is_finite()
            && self.x > -10000.0
            && self.y > -10000.0
        {
            settings.position = window::Position::Specific(Point::new(self.x, self.y));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub font_size: f32,
    pub load_bios: bool,
    pub show_registers: bool,
    pub show_deassembly: bool,
    pub show_memory: bool,
    pub theme: AppTheme,
    pub main_window: Option<WindowGeometry>,
    pub sim_window: Option<WindowGeometry>,
    pub sim_debug_window: Option<WindowGeometry>,
    pub registers_window: Option<WindowGeometry>,
    pub deassembly_window: Option<WindowGeometry>,
    pub memory_window: Option<WindowGeometry>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            load_bios: true,
            show_registers: true,
            show_deassembly: true,
            show_memory: true,
            theme: AppTheme::Dark,
            main_window: None,
            sim_window: None,
            sim_debug_window: None,
            registers_window: None,
            deassembly_window: None,
            memory_window: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    Light,
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
    Ferra,
}

impl AppTheme {
    pub const ALL: [AppTheme; 22] = [
        AppTheme::Light,
        AppTheme::Dark,
        AppTheme::Dracula,
        AppTheme::Nord,
        AppTheme::SolarizedLight,
        AppTheme::SolarizedDark,
        AppTheme::GruvboxLight,
        AppTheme::GruvboxDark,
        AppTheme::CatppuccinLatte,
        AppTheme::CatppuccinFrappe,
        AppTheme::CatppuccinMacchiato,
        AppTheme::CatppuccinMocha,
        AppTheme::TokyoNight,
        AppTheme::TokyoNightStorm,
        AppTheme::TokyoNightLight,
        AppTheme::KanagawaWave,
        AppTheme::KanagawaDragon,
        AppTheme::KanagawaLotus,
        AppTheme::Moonfly,
        AppTheme::Nightfly,
        AppTheme::Oxocarbon,
        AppTheme::Ferra,
    ];
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            AppTheme::Light => "Light",
            AppTheme::Dark => "Dark",
            AppTheme::Dracula => "Dracula",
            AppTheme::Nord => "Nord",
            AppTheme::SolarizedLight => "Solarized Light",
            AppTheme::SolarizedDark => "Solarized Dark",
            AppTheme::GruvboxLight => "Gruvbox Light",
            AppTheme::GruvboxDark => "Gruvbox Dark",
            AppTheme::CatppuccinLatte => "Catppuccin Latte",
            AppTheme::CatppuccinFrappe => "Catppuccin Frappe",
            AppTheme::CatppuccinMacchiato => "Catppuccin Macchiato",
            AppTheme::CatppuccinMocha => "Catppuccin Mocha",
            AppTheme::TokyoNight => "Tokyo Night",
            AppTheme::TokyoNightStorm => "Tokyo Night Storm",
            AppTheme::TokyoNightLight => "Tokyo Night Light",
            AppTheme::KanagawaWave => "Kanagawa Wave",
            AppTheme::KanagawaDragon => "Kanagawa Dragon",
            AppTheme::KanagawaLotus => "Kanagawa Lotus",
            AppTheme::Moonfly => "Moonfly",
            AppTheme::Nightfly => "Nightfly",
            AppTheme::Oxocarbon => "Oxocarbon",
            AppTheme::Ferra => "Ferra",
        };
        write!(f, "{}", label)
    }
}

impl Preferences {
    pub fn load() -> Self {
        let path = prefs_path();
        let contents = std::fs::read_to_string(path);
        contents
            .ok()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = prefs_path();
        if let Ok(text) = toml::to_string_pretty(self) {
            let _ = std::fs::write(path, text);
        }
    }
}

fn prefs_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(PREFERENCES_FILE)
}
