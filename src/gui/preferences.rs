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
    pub main_window: Option<WindowGeometry>,
    pub sim_window: Option<WindowGeometry>,
    pub sim_debug_window: Option<WindowGeometry>,
    pub registers_window: Option<WindowGeometry>,
    pub deassembly_window: Option<WindowGeometry>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            load_bios: true,
            show_registers: true,
            show_deassembly: true,
            main_window: None,
            sim_window: None,
            sim_debug_window: None,
            registers_window: None,
            deassembly_window: None,
        }
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
