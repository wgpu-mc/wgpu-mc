#![allow(dead_code)]

use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

use crate::RUN_DIRECTORY;

static RENDERER_CONFIG_JSON: OnceCell<PathBuf> = OnceCell::new();

/// Add your settings here. Only use the structs from this
/// file, like StringSetting, FloatSetting and IntSetting,
/// then add an appropriate struct to SettingsInfo below,
/// and a default value in the Default impl for this.
#[derive(Serialize, Deserialize)]
#[non_exhaustive]
pub struct Settings {
    pub vsync: BoolSetting,
    pub test_string: StringSetting,
    pub test_float: FloatSetting,
    pub test_int: IntSetting,
}

#[derive(Serialize)]
pub struct SettingsInfo {
    vsync: SettingInfo,
    test_string: SettingInfo,
    test_float: SettingInfo,
    test_int: SettingInfo,
}

const SETTINGS_INFO: SettingsInfo = SettingsInfo {
    vsync: SettingInfo {
        desc: "Whether or not to sync the framerate to the display's framerate.\
        May reduce screen tearing, on the cost of added latency.",
        needs_restart: false,
    },
    test_string: SettingInfo {
        desc: "test string - ignore this",
        needs_restart: false,
    },
    test_float: SettingInfo {
        desc: "test float - ignore this",
        needs_restart: false,
    },
    test_int: SettingInfo {
        desc: "test int - ignore this",
        needs_restart: false,
    },
};

lazy_static! {
    static ref SETTINGS_INFO_JSON: String = serde_json::to_string(&SETTINGS_INFO).unwrap();
}

impl Settings {
    /// Loads the settings from disk, or returns the defaults.
    pub fn load_or_default() -> Settings {
        let config_path = Self::config_path_get_or_init();
        if config_path.exists() {
            let contents = std::fs::read_to_string(config_path).unwrap_or_default();
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            let default = Settings::default();
            default.write();
            default
        }
    }

    fn config_path_get_or_init<'a>() -> &'a PathBuf {
        RENDERER_CONFIG_JSON.get_or_init(|| {
            let mut path = RUN_DIRECTORY.get().unwrap().clone();
            let path_from_dot_minecraft = Path::new("config/fabric/wgpu-mc-renderer.json");
            path.push(path_from_dot_minecraft);
            path
        })
    }

    pub fn write(&self) -> bool {
        let config_path = Self::config_path_get_or_init();

        let str = serde_json::to_string_pretty(self).unwrap();
        std::fs::write(config_path, str).unwrap_or_else(|_| {
            panic!(
                "Couldn't write wgpu-mc-renderer.json (config) to {:?}",
                config_path
            )
        });
        true
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            vsync: BoolSetting { value: true },
            test_string: StringSetting {
                value: "".to_string(),
            },
            test_float: FloatSetting {
                min: Some(-1.0),
                max: None,
                value: 0.0,
            },
            test_int: IntSetting {
                min: Some(0),
                max: Some(100),
                value: 0,
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SettingInfo {
    desc: &'static str,
    needs_restart: bool,
}

impl SettingInfo {
    pub fn get_desc(&self) -> &'static str {
        self.desc
    }

    pub fn get_needs_reload(&self) -> bool {
        self.needs_restart
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename = "bool")]
pub struct BoolSetting {
    pub value: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename = "string")]
pub struct StringSetting {
    pub value: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename = "float")]
pub struct FloatSetting {
    min: Option<f64>,
    max: Option<f64>,
    pub value: f64,
}

impl FloatSetting {
    pub fn get_min(&self) -> Option<f64> {
        self.min
    }

    pub fn get_max(&self) -> Option<f64> {
        self.max
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename = "int")]
pub struct IntSetting {
    min: Option<i64>,
    max: Option<i64>,
    pub value: i64,
}

impl IntSetting {
    pub fn get_min(&self) -> Option<i64> {
        self.min
    }

    pub fn get_max(&self) -> Option<i64> {
        self.max
    }
}
