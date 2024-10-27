use lazy_static::lazy_static;
use std::env;
use std::path::PathBuf;
use crate::functions::to_bool_string;
use serde::{Deserialize, Serialize};
use std::fs;
use eyre::Result;
use eyre::WrapErr;
use log::warn;

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
pub struct JournalctlSummaryProfile {
    pub column_widths: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct JournalctlSummaryMoniker {
    pub column_widths: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct JournalctlSummary {
    pub profile: JournalctlSummaryProfile,
    pub moniker: JournalctlSummaryMoniker,
}

#[derive(Serialize, Deserialize)]
pub struct JournalctlTail {
    pub column_widths: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct JournalctlConfig {
    pub tail: JournalctlTail,
    pub summary: JournalctlSummary,
}

impl Default for JournalctlTail {
    fn default() -> Self {
        Self {
            column_widths: vec![11, 1, 8, 7, 7, 6, 6, 7, 8, 8, 9, 7],
        }
    }
}

impl Default for JournalctlSummaryProfile {
    fn default() -> Self {
        Self {
            column_widths: vec![5, 8, 8],
        }
    }
}

impl Default for JournalctlSummaryMoniker {
    fn default() -> Self {
        Self {
            column_widths: vec![5, 8, 8],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GlobalConfig {
    pub log: LogConfig,
    pub journalctl: JournalctlConfig,
}

/// Provides a default implementation for `GlobalConfig`
/// The default `GlobalConfig` will have the default `LogConfig`
impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            log: LogConfig::default(),
            journalctl: JournalctlConfig {
                tail: JournalctlTail::default(),
                summary: JournalctlSummary {
                    profile: JournalctlSummaryProfile::default(),
                    moniker: JournalctlSummaryMoniker::default(),
                }
            }
        }
    }
}


impl GlobalConfig {

    // Path to the config file
    pub fn path() -> PathBuf {
        let default_filename = "config.toml";
        let filename = match env::current_exe() {
            Ok(path) => match path.file_name() {
                Some(name) => format!("{}.toml", name.to_string_lossy()),
                None => {
                    warn!("Failed to get the filename from the path");
                    default_filename.to_string()
                }
            },
            Err(e) => {
                warn!("Failed to get the current executable path: {}", e);
                default_filename.to_string()
            },
        };
        PROFILES_DIR.join(filename)
    }

    /// Creates a new `GlobalConfig` with default values, identical to `default()`.
    pub fn new() -> Self {
        Self::default() // Delegate to `Default` implementation
    }

    // Load configuration from the TOML file
    pub fn load() -> Result<Self> {
        let config_path = Self::path();
        let config_str = fs::read_to_string(&config_path)
            .wrap_err_with(|| format!("Failed to read config file at {:?}", config_path))?;
        toml::from_str(&config_str)
            .wrap_err("Failed to parse config TOML")
    }

    // Save the current configuration to the TOML file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::path();
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
            self.save() // Save the updated config to disk
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
