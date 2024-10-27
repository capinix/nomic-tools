use lazy_static::lazy_static;
use std::env;
use std::path::PathBuf;
//use crate::functions::to_bool_string;
use serde::{Deserialize, Serialize};
use std::fs;
use eyre::Result;
use eyre::WrapErr;
use log::warn;
use clap::ValueEnum;

#[derive(Clone, Debug, ValueEnum)]
pub enum GroupBy {
    Profile,
    Moniker,
}


#[derive(Clone, Deserialize, Serialize)]
pub struct JournalctlSummaryProfile {
    pub column_widths: Vec<usize>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct JournalctlSummaryMoniker {
    pub column_widths: Vec<usize>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct JournalctlSummary {
    pub profile: JournalctlSummaryProfile,
    pub moniker: JournalctlSummaryMoniker,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct JournalctlTail {
    pub column_widths: Vec<usize>,
}

#[derive(Clone, Deserialize, Serialize)]
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

#[derive(Clone, Deserialize, Serialize)]
pub struct GlobalConfig {
    pub minimum_balance: u64,
    pub minimum_balance_ratio: u64,
    pub minimum_stake: u64,
    pub adjust_minimum_stake: bool,
    pub minimum_stake_rounding: u64,
    pub claim_fee: u64,
    pub stake_fee: u64,
    pub nomic_legacy_version: String,
    pub nomic_exe: PathBuf,
    pub journalctl: JournalctlConfig,
}

/// Provides a default implementation for `GlobalConfig`
/// The default `GlobalConfig` will have the default `LogConfig`
impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            minimum_balance: 100_000,
            minimum_balance_ratio: 1_000,
            minimum_stake: 1_000_000,
            adjust_minimum_stake: false,
            minimum_stake_rounding: 100_000,
            claim_fee: 10_000,
            stake_fee: 10_000,
            nomic_legacy_version: "NOMIC_LEGACY_VERSION=".to_string(),
            nomic_exe: PathBuf::from("/usr/local/bin/nomic"),
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
            Ok(path) => path.file_stem()
                .map(|name| format!("{}.toml", name.to_string_lossy()))
                .unwrap_or_else(|| default_filename.to_string()),
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

    // Load configuration from the TOML file and override with environment variables
    pub fn load() -> Self {
        let config_path = Self::path();

        // Try to load from file
        let mut config = fs::read_to_string(&config_path)
            .wrap_err_with(|| format!("Failed to read config file at {:?}", config_path))
            .and_then(|config_str| {
                toml::from_str(&config_str).wrap_err("Failed to parse config TOML")
            })
            .unwrap_or_else(|err| {
                warn!("Error loading config: {}. Using defaults.", err);
                Self::default() // Use default on error
            });

        // Override with environment variables if they exist
        if let Ok(val) = env::var("MINIMUM_BALANCE") {
            config.minimum_balance = val.parse().unwrap_or(config.minimum_balance);
        }
        if let Ok(val) = env::var("MINIMUM_BALANCE_RATIO") {
            config.minimum_balance_ratio = val.parse().unwrap_or(config.minimum_balance_ratio);
        }
        if let Ok(val) = env::var("MINIMUM_STAKE") {
            config.minimum_stake = val.parse().unwrap_or(config.minimum_stake);
        }
        if let Ok(val) = env::var("ADJUST_MINIMUM_STAKE") {
            config.adjust_minimum_stake = val.parse().unwrap_or(config.adjust_minimum_stake);
        }
        if let Ok(val) = env::var("MINIMUM_STAKE_ROUNDING") {
            config.minimum_stake_rounding = val.parse().unwrap_or(config.minimum_stake_rounding);
        }
        if let Ok(val) = env::var("CLAIM_FEE") {
            config.claim_fee = val.parse().unwrap_or(config.claim_fee);
        }
        if let Ok(val) = env::var("STAKE_FEE") {
            config.stake_fee = val.parse().unwrap_or(config.stake_fee);
        }
        if let Ok(val) = env::var("NOMIC_LEGACY_VERSION") {
            config.nomic_legacy_version = val;
        }
        if let Ok(val) = env::var("NOMIC") {
            config.nomic_exe = PathBuf::from(val);
        }

        config
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
    pub fn set_journalctl_tail_column_width(&mut self, column: usize, width: usize) -> Result<()> {
        if column < self.journalctl.tail.column_widths.len() {
            self.journalctl.tail.column_widths[column] = width; // Update the specified column width
            self.save() // Save the updated config to disk
        } else {
            Err(eyre::eyre!("Column index out of bounds")) // Handle out-of-bounds index
        }
    }

    // Edit a specific log column width and save to disk
    pub fn set_journalctl_summary_column_width(&mut self, group: GroupBy, column: usize, width: usize) -> Result<()> {
        let column_widths = match group {
            GroupBy::Profile => &mut self.journalctl.summary.profile.column_widths,
            GroupBy::Moniker => &mut self.journalctl.summary.moniker.column_widths,
        };

        if column < column_widths.len() {
            // Update the specified column width
            column_widths[column] = width;

            // Save the updated config to disk
            self.save().wrap_err("Failed to save updated configuration")
        } else {
            Err(eyre::eyre!("Column index out of bounds for {:?}", group)) // Handle out-of-bounds index
        }
    }

}

// Immutable global configuration
lazy_static! {
    pub static ref CONFIG: GlobalConfig = GlobalConfig::load();

    pub static ref PROFILES_DIR: PathBuf = {
        // Check for environment variable
        if let Ok(env_dir) = env::var("PROFILES_DIR") {
            PathBuf::from(env_dir)
        } else {
            // Default to $HOME/.nomic-tools if not set
            match home::home_dir() {
                Some(home_dir) => home_dir.join(".nomic-tools"),
                None => {
                    warn!("HOME directory could not be determined. Using current directory as fallback.");
                    PathBuf::from(".nomic-tools") // Fallback to current directory
                }
            }
        }
    };
}
