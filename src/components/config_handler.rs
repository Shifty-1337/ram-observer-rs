use std::{
    fs, 
    io::{self, Error, ErrorKind}, 
    path::Path
};
use serde_json;

use crate::components::{
    constants::{DEFAULT_AUTO_THRESHOLD, CONFIG_FILE},
    structs::{Config, RamMonitor},
};

type ConfigResult<T> = io::Result<T>;
type ValidationMessage = (String, bool);

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_threshold: DEFAULT_AUTO_THRESHOLD,
            auto_action: String::from("Empty Working Sets"),
        }
    }
}

impl Config {
    fn is_valid_threshold(threshold: f32) -> bool {
        threshold >= 20.0 && threshold <= 95.0
    }

    fn is_valid_action(action: &str) -> bool {
        matches!(
            action,
            "Empty Working Sets" |
            "Empty System Working Sets" |
            "Empty Modified Page Lists" |
            "Empty Standby List" |
            "Empty Priority 0 Standby List"
        )
    }

    fn validate_config(config: &mut Config) -> Vec<ValidationMessage> {
        let mut messages = Vec::new();
        
        if !Self::is_valid_threshold(config.auto_threshold) {
            messages.extend([
                (format!("Invalid threshold value {}, using default", config.auto_threshold), true),
                (format!("Using default threshold: {}%", DEFAULT_AUTO_THRESHOLD), false),
            ]);
            config.auto_threshold = DEFAULT_AUTO_THRESHOLD;
        }

        if !Self::is_valid_action(&config.auto_action) {
            messages.extend([
                (format!("Invalid action {}, using default", config.auto_action), true),
                (format!("Using default action: Empty Working Sets"), false),
            ]);
            config.auto_action = String::from("Empty Working Sets");
        }

        messages
    }

    pub fn load(ram_monitor: &mut RamMonitor) -> Self {
        if !Path::new(CONFIG_FILE).exists() {
            return Config::default();
        }

        fs::read_to_string(CONFIG_FILE)
            .map_err(|e| {
                ram_monitor.add_log(
                    format!("Error reading config file: {}, using defaults", e),
                    true
                );
                e
            })
            .and_then(|contents| {
                serde_json::from_str(&contents).map_err(|e| {
                    ram_monitor.add_log(
                        format!("Error parsing config file: {}, using defaults", e),
                        true
                    );
                    Error::new(ErrorKind::InvalidData, e)
                })
            })
            .map(|mut config| {
                let messages = Self::validate_config(&mut config);
                for msg in messages {
                    ram_monitor.add_log(msg.0, msg.1);
                }
                config
            })
            .unwrap_or_else(|_| Config::default())
    }

    pub fn save(&self) -> ConfigResult<Vec<ValidationMessage>> {
        let mut config = self.clone();
        let messages = Self::validate_config(&mut config);
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(CONFIG_FILE, config_json)?;
        Ok(messages)
    }
}