
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::path::Path;
use toml;
use eyre::Result;
use eyre::WrapErr;
use crate::functions::format_to_millions;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConfigValidator {
    pub address: String,
    pub name: String,
}

// Implement Display for ConfigValidator
impl std::fmt::Display for ConfigValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.address, self.name)
    }
}

impl ConfigValidator {
    pub fn new(address: &str, name: &str) -> Self {
        Self {
            address: address.to_string(), 
            name:    name.to_string()
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub profile:                  String,
    pub minimum_balance:             u64,
    pub minimum_balance_ratio:       u64,
    pub minimum_stake:               u64,
    pub adjust_minimum_stake:       bool,
    pub minimum_stake_rounding:      u64,
    pub daily_reward:                u64,
    pub validators: Vec<ConfigValidator>,
}

// Implement Display for Config
impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Format the profile and other fields
        writeln!(f, "{:22} = {}", "Profile", self.profile )?;
        writeln!(f, "{:22} = {}", "Minimum Balance", format_to_millions(self.minimum_balance))?;
        writeln!(f, "{:22} = {}", "Minimum Balance Ratio", format_to_millions(self.minimum_balance_ratio))?;
        writeln!(f, "{:22} = {}", "Minimum Stake", format_to_millions(self.minimum_stake))?;
        writeln!(f, "{:22} = {}", "Adjust Minimum Stake", self.adjust_minimum_stake)?;
        writeln!(f, "{:22} = {}", "Minimum Stake Rounding", format_to_millions(self.minimum_stake_rounding))?;
        writeln!(f, "{:22} = {}", "Daily Reward", self.daily_reward)?;

        // Format the validators
        writeln!(f, "Validators:")?;
        for validator in &self.validators {
            writeln!(f, "  - {}", validator)?; // Using Display implementation of ConfigValidator
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            profile:           String::new(), // Empty profile by default
            minimum_balance:          10_000, // Default balance
            minimum_balance_ratio:     1_000, // Default to 0.001 (as f64 divided by 1_000_000.0)
            minimum_stake:         1_000_000, // Default minimum stake
            adjust_minimum_stake:      false, // Default adjustment to false
            minimum_stake_rounding:   10_000, // Default rounding
            daily_reward:                  0, // Default daily reward is zero
            validators:           Vec::new(), // Start with no validators
        }
    }
}

impl Config {

    pub fn new(profile: &str) -> Self {
        let mut config = Config::default();
        config.profile = profile.to_string();
        config
    }

    pub fn add_validator(&mut self, address: &str, name: &str) {
        let validator = ConfigValidator::new(address, name);
        self.validators.push(validator);
    }

    /// Save the current configuration to a TOML file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let toml_str = toml::to_string(self)
            .wrap_err("Failed to serialize config to TOML")?;
        fs::write(path, toml_str)
            .wrap_err_with(|| format!("Failed to write config file at {:?}", path))?;
        Ok(())
    }

    /// Load the configuration from a TOML file.
    pub fn load(path: &Path) -> Result<Self> {
        let config_str = fs::read_to_string(path)
            .wrap_err_with(|| format!("Failed to read config file at {:?}", path))?;
        let config: Config = toml::from_str(&config_str)
            .wrap_err("Failed to parse config TOML")?;
        Ok(config)
    }

    pub fn active_validator(&self) -> Result<&ConfigValidator> {
        match self.validators.last() {
            Some(v) => Ok(v),
            None => Err(eyre::eyre!("No validators found")),
        }
    }

    pub fn search_validator(&self, search: &str) -> Result<&ConfigValidator> {
        // Convert the search string to lowercase for case-insensitive comparison
        let search_lower = search.to_lowercase();

        // Find the validator where either the address or name matches the search term
        self.validators
            .iter()
            .find(|validator| {
                validator.address.to_lowercase() == search_lower ||
                validator.name.to_lowercase() == search_lower
            })
            .ok_or_else(|| eyre::eyre!("Validator not found"))
    }

    pub fn remove_validator(&mut self, search: &str) -> Result<&mut Self> {
        // Convert search to lowercase for case-insensitive comparison
        let search_lower = search.to_lowercase();

        // Retain only validators that do not match the search criteria
        self.validators.retain(|validator| {
            validator.address.to_lowercase() != search_lower
                && validator.name.to_lowercase() != search_lower
        });

        Ok(self)
    }

    pub fn rotate_validators(&mut self) -> &mut Self {
        // Rotate the validators if the list is not empty
        if let Some(last) = self.validators.pop() {
            self.validators.insert(0, last);
        }
        self
    }
}
