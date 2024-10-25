use lazy_static::lazy_static;
use std::env;
use std::path::PathBuf;
use crate::functions::to_bool_string;
use serde::{Deserialize, Serialize};
use std::fs;
use eyre::Result;
use eyre::WrapErr;

#[derive(Serialize, Deserialize)]
pub struct LogConfig {
    pub column_widths: Vec<usize>,
}

/// Provides a default implementation for `LogConfig`.
/// The default `LogConfig` will have an empty `column_widths` vector.
impl Default for LogConfig {
    fn default() -> Self {
        Self {
            column_widths: vec![11, 1, 8, 7, 7, 6, 6, 7, 8, 8, 9, 7],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GlobalConfig {
    pub log: LogConfig,
}

/// Provides a default implementation for `GlobalConfig`.
/// The default `GlobalConfig` will have the default `LogConfig`.
impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            log: LogConfig::default(), // Use the default `LogConfig`
        }
    }
}


impl GlobalConfig {
    /// Creates a new `GlobalConfig` with default values, identical to `default()`.
    pub fn new() -> Self {
        Self::default() // Delegate to `Default` implementation
    }

    // Path to the config file
    fn config_file_path() -> PathBuf {
        PROFILES_DIR.join("config.toml")
    }

    // Load configuration from the TOML file
    pub fn load_config() -> Result<Self> {
        let config_path = Self::config_file_path();
        let config_str = fs::read_to_string(&config_path)
            .wrap_err_with(|| format!("Failed to read config file at {:?}", config_path))?;
        toml::from_str(&config_str)
            .wrap_err("Failed to parse config TOML")
    }

    // Save the current configuration to the TOML file
    pub fn save_config(&self) -> Result<()> {
        let config_path = Self::config_file_path();
        let toml_str = toml::to_string(self)
            .wrap_err("Failed to serialize config to TOML")?;
        fs::write(&config_path, toml_str)
            .wrap_err_with(|| format!("Failed to write config to {:?}", config_path))?;
        Ok(())
    }

    // Edit a specific log column width and save to disk
    pub fn set_log_column_width(&mut self, column: usize, width: usize) -> Result<()> {
        if column < self.log.column_widths.len() {
            self.log.column_widths[column] = width; // Update the specified column width
            self.save_config() // Save the updated config to disk
        } else {
            Err(eyre::eyre!("Column index out of bounds")) // Handle out-of-bounds index
        }
    }

}


lazy_static! {

    pub static ref NOMIC_LEGACY_VERSION: String = env::var("NOMIC_LEGACY_VERSION").ok()
        .unwrap_or(String::new());

    pub static ref MINIMUM_BALANCE: Option<f64> = env::var("MINIMUM_BALANCE")
        .ok()
        .and_then(|val| val.parse::<f64>().ok());

    pub static ref MINIMUM_BALANCE_RATIO: Option<f64> = env::var("MINIMUM_BALANCE_RATIO")
        .ok()
        .and_then(|val| val.parse::<f64>().ok());

    pub static ref MINIMUM_STAKE: Option<f64> = env::var("MINIMUM_STAKE")
        .ok()
        .and_then(|val| val.parse::<f64>().ok());

    pub static ref ADJUST_MINIMUM_STAKE: Option<String> = env::var("ADJUST_MINIMUM_STAKE")
        .ok()
        .and_then(|val| to_bool_string(val));

    pub static ref MINIMUM_STAKE_ROUNDING: Option<f64> = env::var("MINIMUM_STAKE_ROUNDING")
        .ok()
        .and_then(|val| val.parse::<f64>().ok());

    pub static ref CLAIM_FEE: f64 = env::var("CLAIM_FEE")
        .ok()
        .and_then(|val| val.parse::<f64>().ok())
        .unwrap_or(0.01);

    pub static ref STAKE_FEE: f64 = env::var("STAKE_FEE")
        .ok()
        .and_then(|val| val.parse::<f64>().ok())
        .unwrap_or(0.01);

    pub static ref PROFILES_DIR: PathBuf = {
        // Check for environment variable
        if let Ok(env_dir) = env::var("PROFILES_DIR") {
            PathBuf::from(env_dir)
        } else {
            // Default to $HOME/.nomic-tools if not set
            let home_dir = env::var("HOME").expect("Failed to get HOME environment variable");
            PathBuf::from(home_dir).join(".nomic-tools")
        }
    };

    pub static ref NOMIC: String = {
        env::var("NOMIC").unwrap_or_else(|_| String::from("/usr/local/bin/nomic"))
    };

}
