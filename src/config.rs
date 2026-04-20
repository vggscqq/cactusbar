use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldClock {
    pub name: String,
    pub zone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageEntry {
    pub r#match: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modules {
    #[serde(default)]
    pub workspaces: bool,
    #[serde(default = "default_true")]
    pub focused_app: bool,
    #[serde(default = "default_true")]
    pub music: bool,
    #[serde(default = "default_true")]
    pub mode: bool,
    #[serde(default = "default_true")]
    pub scratchpad: bool,
    #[serde(default = "default_true")]
    pub date_clock: bool,
    #[serde(default = "default_true")]
    pub time_clock: bool,
    #[serde(default = "default_true")]
    pub notification: bool,
    #[serde(default = "default_true")]
    pub mpd: bool,
    #[serde(default = "default_true")]
    pub wallpaper: bool,
    #[serde(default = "default_true")]
    pub clipboard: bool,
    #[serde(default = "default_true")]
    pub weather: bool,
    #[serde(default = "default_true")]
    pub pipewire: bool,
    #[serde(default = "default_true")]
    pub network: bool,
    #[serde(default = "default_true")]
    pub bluetooth: bool,
    #[serde(default)]
    pub power_profile: bool,
    #[serde(default = "default_true")]
    pub cpu: bool,
    #[serde(default = "default_true")]
    pub memory: bool,
    #[serde(default = "default_true")]
    pub temperature: bool,
    #[serde(default)]
    pub keyboard_state: bool,
    #[serde(default = "default_true")]
    pub keyboard_layout: bool,
    #[serde(default = "default_true")]
    pub language: bool,
    #[serde(default = "default_true")]
    pub battery: bool,
    #[serde(default = "default_true")]
    pub tray: bool,
    #[serde(default = "default_true")]
    pub power: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Modules {
    fn default() -> Self {
        Self {
            workspaces: false,
            focused_app: true,
            music: true,
            mode: true,
            scratchpad: true,
            date_clock: true,
            time_clock: true,
            notification: true,
            mpd: true,
            wallpaper: true,
            clipboard: true,
            weather: true,
            pipewire: true,
            network: true,
            bluetooth: true,
            power_profile: false,
            cpu: true,
            memory: true,
            temperature: true,
            keyboard_state: false,
            keyboard_layout: true,
            language: true,
            battery: true,
            tray: true,
            power: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WeatherConfig {
    #[serde(default)]
    pub lat: String,
    #[serde(default)]
    pub lon: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub on_click: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WallpaperConfig {
    #[serde(default)]
    pub dir: String,
    #[serde(default)]
    pub auto_switch: bool,
    #[serde(default)]
    pub interval: i64,
    #[serde(default)]
    pub on_click: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioConfig {
    #[serde(default)]
    pub on_click: String,
    #[serde(default)]
    pub show_text: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkConfig {
    #[serde(default)]
    pub on_click: String,
    #[serde(default)]
    pub show_text: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BluetoothConfig {
    #[serde(default)]
    pub show_text: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClocksConfig {
    #[serde(default)]
    pub on_click: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalendarConfig {
    #[serde(default)]
    pub on_click: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatteryConfig {
    #[serde(default)]
    pub show_text: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FocusedAppConfig {
    #[serde(default)]
    pub show_empty_workspace: bool,
    #[serde(default)]
    pub empty_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub modules: Modules,
    #[serde(default)]
    pub world_clocks: Vec<WorldClock>,
    #[serde(default)]
    pub wallpaper: WallpaperConfig,
    #[serde(default)]
    pub weather: WeatherConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub bluetooth: BluetoothConfig,
    #[serde(default)]
    pub clocks: ClocksConfig,
    #[serde(default)]
    pub calendar: CalendarConfig,
    #[serde(default)]
    pub battery_config: BatteryConfig,
    #[serde(default)]
    pub focused_app_config: FocusedAppConfig,
    #[serde(default)]
    pub languages: Vec<LanguageEntry>,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs_home();
        Self {
            modules: Modules::default(),
            world_clocks: vec![
                WorldClock { name: "Prague".into(), zone: "Europe/Prague".into() },
                WorldClock { name: "New York".into(), zone: "America/New_York".into() },
                WorldClock { name: "Tel Aviv".into(), zone: "Asia/Tel_Aviv".into() },
                WorldClock { name: "Kyiv".into(), zone: "Europe/Kyiv".into() },
                WorldClock { name: "San Francisco".into(), zone: "America/Los_Angeles".into() },
                WorldClock { name: "Maui".into(), zone: "Pacific/Honolulu".into() },
            ],
            wallpaper: WallpaperConfig {
                dir: format!("{}/Pictures/wp", home),
                auto_switch: true,
                interval: 10,
                on_click: String::new(),
            },
            weather: WeatherConfig {
                lat: "50.0755".into(),
                lon: "14.4378".into(),
                location: "Prague".into(),
                on_click: "gnome-weather".into(),
            },
            audio: AudioConfig {
                on_click: "flatpak run com.saivert.pwvucontrol".into(),
                show_text: false,
            },
            network: NetworkConfig {
                on_click: "nm-connection-editor".into(),
                show_text: false,
            },
            bluetooth: BluetoothConfig { show_text: false },
            clocks: ClocksConfig { on_click: "gnome-clocks".into() },
            calendar: CalendarConfig { on_click: "gnome-calendar".into() },
            battery_config: BatteryConfig { show_text: true },
            focused_app_config: FocusedAppConfig {
                show_empty_workspace: true,
                empty_text: "Desktop".into(),
            },
            languages: vec![],
        }
    }
}

fn dirs_home() -> String {
    std::env::var("HOME").unwrap_or_else(|_| {
        dirs_home_fallback()
    })
}

fn dirs_home_fallback() -> String {
    if let Ok(h) = std::env::var("HOME") {
        return h;
    }
    "/root".to_string()
}

fn config_path() -> PathBuf {
    if let Ok(p) = std::env::var("STATUSBAR_CONFIG") {
        return PathBuf::from(p);
    }
    let home = dirs_home();
    PathBuf::from(format!("{}/.config/cactusbar/config.yaml", home))
}

pub fn load() -> Config {
    let mut cfg = Config::default();
    let path = config_path();

    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return cfg,
        Err(e) => {
            log::warn!("config: read {:?}: {}", path, e);
            return cfg;
        }
    };

    match serde_yaml::from_str::<Config>(&data) {
        Ok(parsed) => cfg = parsed,
        Err(e) => log::warn!("config: parse {:?}: {}", path, e),
    }

    // Expand ~ in wallpaper dir
    if cfg.wallpaper.dir.starts_with("~/") {
        let home = dirs_home();
        cfg.wallpaper.dir = format!("{}/{}", home, &cfg.wallpaper.dir[2..]);
    }
    if cfg.wallpaper.dir.is_empty() {
        cfg.wallpaper.dir = format!("{}/Pictures/wp", dirs_home());
    }

    cfg
}
