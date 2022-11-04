#![allow(dead_code)]

use std::path::{Path, PathBuf};

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

use crate::RUN_DIRECTORY;

/// Add your settings here. Only use the structs from this
/// file, like StringSetting, FloatSetting and IntSetting,
/// then add default values in the Default impl below.
#[derive(Deserialize, Serialize)]
#[non_exhaustive]
pub struct Settings {
    pub test_string: StringSetting,
    pub test_float: FloatSetting,
    pub test_int: IntSetting,
}

static CONFIG_PATH: OnceCell<PathBuf> = OnceCell::new();

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
        CONFIG_PATH.get_or_init(|| {
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
            test_string: StringSetting {
                generic: GenericSetting {
                    name: "testString".to_string(),
                    desc: "my description for test string".to_string(),
                    needs_reload: false,
                },
                value: "default_value_for_this".to_string(),
            },
            test_float: FloatSetting {
                generic: GenericSetting {
                    name: "testFloat".to_string(),
                    desc: "my desc".to_string(),
                    needs_reload: false,
                },
                min: 0.0,
                max: 1.0,
                value: 0.5,
            },
            test_int: IntSetting {
                generic: GenericSetting {
                    name: "test_int".to_string(),
                    desc: "".to_string(),
                    needs_reload: false,
                },
                min: -1,
                max: 1,
                value: 0,
            },
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct GenericSetting {
    name: String,
    desc: String,
    needs_reload: bool,
}

impl GenericSetting {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_desc(&self) -> &String {
        &self.desc
    }

    pub fn get_needs_reload(&self) -> bool {
        self.needs_reload
    }
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename = "string")]
pub struct StringSetting {
    #[serde(flatten)]
    generic: GenericSetting,
    pub value: String,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename = "float")]
pub struct FloatSetting {
    #[serde(flatten)]
    generic: GenericSetting,
    min: f64,
    max: f64,
    pub value: f64,
}

impl FloatSetting {
    pub fn get_min(&self) -> f64 {
        self.min
    }

    pub fn get_max(&self) -> f64 {
        self.max
    }
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename = "int")]
pub struct IntSetting {
    #[serde(flatten)]
    generic: GenericSetting,
    min: i64,
    max: i64,
    pub value: i64,
}

impl IntSetting {
    pub fn get_min(&self) -> i64 {
        self.min
    }

    pub fn get_max(&self) -> i64 {
        self.max
    }
}
