
use crate::functions::grep_config;
use crate::functions::to_bool;
use crate::globals::{
    ADJUST_MINIMUM_STAKE, 
    MINIMUM_BALANCE, 
    MINIMUM_BALANCE_RATIO, 
    MINIMUM_STAKE, 
    MINIMUM_STAKE_ROUNDING, 
};
use crate::validators::ValidatorCollection;
use crate::validators::initialize_validators;
use eyre::WrapErr;
use eyre::Result;
use once_cell::sync::OnceCell;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use chrono::{Utc, DateTime};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConfigValidator {
    pub address: String,
    pub moniker: String,
}

impl ConfigValidator {
    pub fn new(address: String, moniker: String) -> Self {
        Self { address, moniker }
    }
}

#[derive(Clone)]
pub struct Config {
    profile:                           Option<String>,
    content:                                   String,
    minimum_balance:                    OnceCell<u64>,
    minimum_balance_ratio:              OnceCell<f64>,
    minimum_stake:                      OnceCell<u64>,
    adjust_minimum_stake:              OnceCell<bool>,
    minimum_stake_rounding:             OnceCell<u64>,
    daily_reward:                       OnceCell<u64>,
    config_validators: OnceCell<Vec<ConfigValidator>>,
    active_validator:       OnceCell<ConfigValidator>,
    validators:         OnceCell<ValidatorCollection>,
    #[allow(dead_code)]
    timestamp:                          DateTime<Utc>,
}

impl Config {

    pub fn set_minimum_balance(&mut self, balance: u64) -> &mut Self {
        self.minimum_balance = OnceCell::from(balance);
        self
    }

    pub fn set_minimum_balance_ratio(&mut self, ratio: f64) -> &mut Self {
        self.minimum_balance_ratio = OnceCell::from(ratio);
        self
    }

    pub fn set_minimum_stake(&mut self, stake: u64) -> &mut Self {
        self.minimum_stake = OnceCell::from(stake);
        self
    }

    pub fn set_adjust_minimum_stake(&mut self, adjust: bool) -> &mut Self {
        self.adjust_minimum_stake = OnceCell::from(adjust);
        self
    }

    pub fn set_minimum_stake_rounding(&mut self, rounding: u64) -> &mut Self {
        self.minimum_stake_rounding = OnceCell::from(rounding);
        self
    }

    pub fn set_daily_reward(&mut self, reward: u64) -> &mut Self {
        self.daily_reward = OnceCell::from(reward);
        self
    }

    pub fn new<P: AsRef<Path>>(
        profile: Option<&str>,
        path: P,
        validators: Option<ValidatorCollection>,
    ) -> Result<Self> {
        let timestamp = Utc::now(); // Current timestamp

        let path = path.as_ref();

        // Attempt to read file content; log an error if it fails
        let content = fs::read_to_string(path)
            .unwrap_or_else(|err| {
                eprintln!("Failed to read from file {:?}: {}", path, err);
                String::new() // Return an empty string if reading fails
            });

        Ok(Self {
            profile: profile.map(|p| p.to_string()),
            content,
            minimum_balance:        OnceCell::new(),
            minimum_balance_ratio:  OnceCell::new(),
            minimum_stake:          OnceCell::new(),
            adjust_minimum_stake:   OnceCell::new(),
            minimum_stake_rounding: OnceCell::new(),
            daily_reward:           OnceCell::new(),
            config_validators:      OnceCell::new(),
            active_validator:       OnceCell::new(),
            validators:             initialize_validators(validators),
            timestamp,
        })
    }

    pub fn minimum_balance(&self) -> &u64 {
        self.minimum_balance.get_or_init(|| {
            // Check the evironment variable
            if let Some(balance) = *MINIMUM_BALANCE {
                return (balance * 1_000_000.0) as u64;
            }

            // Check the configuration file
            if let Some(matched_value) = grep_config("MINIMUM_BALANCE", &self.content) {
                // Multiply by 1,000,000 before converting to u64
                if let Ok(balance) = matched_value.parse::<f64>() {
                    return (balance * 1_000_000.0) as u64;
                }
            }

            // Default value if none found
            100_000
        })
    }

    pub fn minimum_balance_ratio(&self) -> &f64 {
        self.minimum_balance_ratio.get_or_init(|| {
            // Check the evironment variable
            if let Some(ratio) = *MINIMUM_BALANCE_RATIO {
                return ratio;
            }

            // Check the configuration file
            if let Some(matched_value) = grep_config("MINIMUM_BALANCE_RATIO", &self.content) {
                if let Ok(ratio) = matched_value.parse::<f64>() {
                    return ratio;
                }
            }

            // Default value if none found
            0.001 // Default ratio value
        })
    }

    pub fn minimum_stake(&self) -> &u64 {
        self.minimum_stake.get_or_init(|| {
            // Check the evironment variable
            if let Some(stake) = &*MINIMUM_STAKE {
                return (stake * 1_000_000.0) as u64;
            }

            // Check the configuration file
            if let Some(matched_value) = grep_config("MINIMUM_STAKE", &self.content) {
                if let Ok(stake) = matched_value.parse::<f64>() {
                    return (stake * 1_000_000.0) as u64;
                }
            }

            // Default value if none found
            1_000_000
        })
    }

    pub fn adjust_minimum_stake(&self) -> &bool {
        self.adjust_minimum_stake.get_or_init(|| {
            // Check the environment variable
            if let Some(adjust) = &*ADJUST_MINIMUM_STAKE {
                // Convert the value to a bool using to_bool function
                if let Some(min) = to_bool(adjust.to_string()) {
                    return min;
                }
            }

            // Check the configuration file
            if let Some(adjust) = grep_config("ADJUST_MINIMUM_STAKE", &self.content) {
                // Convert the value to a bool using to_bool function
                if let Some(min) = to_bool(adjust) {
                    return min;
                }
            }

            // Default value if none found
            false
        })
    }

    pub fn minimum_stake_rounding(&self) -> &u64 {
        self.minimum_stake_rounding.get_or_init(|| {
            // Check the environment variable
            if let Some(round) = *MINIMUM_STAKE_ROUNDING {
                // Multiply by 1,000,000 and convert to u64
                return (round * 1_000_000.0) as u64;
            }

            // Check the configuration file
            if let Some(matched_value) = grep_config("MINIMUM_STAKE_ROUNDING", &self.content) {
                // Multiply by 1,000,000 before converting to u64
                if let Ok(round) = matched_value.parse::<f64>() {
                    return (round * 1_000_000.0) as u64;
                }
            }

            // Default value if none found
            10_000
        })
    }

    pub fn daily_reward(&self) -> &u64 {
        self.daily_reward.get_or_init(|| {

            // Check the configuration file
            if let Some(matched_value) = grep_config("DAILY_REWARD", &self.content) {
                if let Ok(reward) = matched_value.parse::<f64>() {
                    return (reward * 1_000_000.0) as u64;
                }
            }

            // Default value if none found
            0 // Default reward value
        })
    }

    /// Retrieves validators, initializing it if necessary.
    /// blockchain operation, cache with oncecell
    pub fn validators(&self) -> eyre::Result<&ValidatorCollection> {
        self.validators.get_or_try_init(|| {
            ValidatorCollection::fetch()
        })
    }

    // Getter for config_validators with lazy initialization
    pub fn config_validators(&self) -> eyre::Result<&Vec<ConfigValidator>> {
        self.config_validators.get_or_try_init(|| {
            let mut config_validators = Vec::new();
            let read_validator_regex = Regex::new(concat!(
                r"(?m)^[[:space:]]*read[[:space:]]+(?:-r[[:space:]]+)?",
                r"VALIDATOR[[:space:]]+MONIKER[[:space:]]+<<<[[:space:]]*",
                r#""([^"\\]*)[[:space:]]+([^"\\]*)""#
            )).unwrap();

            for captures in read_validator_regex.captures_iter(&self.content) {
                config_validators.push(ConfigValidator {
                    address: captures.get(1).map_or_else(String::new, |m| m.as_str().to_owned()),
                    moniker: captures.get(2).map_or_else(String::new, |m| m.as_str().to_owned()),
                });
            }

            // If no validators were found in the content, pick random ones
            if config_validators.is_empty() {
                config_validators = self.validators()?.random_percent(2, 20.0, 10.0)?
                    .iter()
                    .map(|validator| ConfigValidator::new(
                        validator.address().to_string(),
                        validator.moniker().to_string()
                    ))
                    .collect();
            }

            // Return the collected config validators
            Ok(config_validators)
        })
    }

    pub fn active_validator(&self) -> eyre::Result<&ConfigValidator> {
        self.active_validator.get_or_try_init(|| {
            if let Some(validator) = self.config_validators()?.last() {
                Ok(validator.clone())
            } else {
                Err(eyre::eyre!("No validators found"))
            }
        })
    }

    pub fn search_validator(&self, search: &str) -> eyre::Result<&str> {

        // Convert the search string to lowercase for case-insensitive comparison
        let search_lower = search.to_lowercase();

        // Find the validator where either the address or moniker matches the search term
        self.config_validators()?
            .iter()
            .find(|validator| {
                validator.address.to_lowercase() == search_lower || 
                validator.moniker.to_lowercase() == search_lower
            })
            .map(|validator| validator.address.as_str())
            .ok_or_else(|| eyre::eyre!("Validator not found"))
    }

    pub fn remove_validator(&mut self, search: &str) -> eyre::Result<&mut Self> {
        // Clone the existing validators and filter them
        let mut validators = self.config_validators()?.clone();

        // Retain only the validators that don't match the search criteria
        validators.retain(|validator| {
            !(validator.address == search || validator.moniker == search)
        });

        // Update the internal state with the new list of validators.
        self.config_validators = OnceCell::from(validators);

        Ok(self)
    }

    pub fn rotate_validators(&mut self) -> eyre::Result<&mut Self> {
        // Access the config_validators (ensure it's initialized)
        let validators = self.config_validators()?; // This returns a Result<&Vec<ConfigValidator>>

        // Create a copy of the existing validators
        let mut validators: Vec<ConfigValidator> = validators.clone();

        // Rotate the validators if the list is not empty
        if let Some(last) = validators.pop() {
            validators.insert(0, last);
        }

        // Update the internal state with the new list of validators.
        self.config_validators = OnceCell::from(validators);

        Ok(self)
    }

    pub fn export(&self) -> eyre::Result<String> {
        let mut output = String::new();

        // Add profile information, handle potential None value
        output.push_str(&format!("PROFILE={}\n",self.profile.as_deref().unwrap_or("unspecified")));

        // Add financial parameters
        output.push_str(&format!(
                "MINIMUM_BALANCE={:.2}\n",
                *self.minimum_balance() as f64 / 1_000_000.0)
        );
        output.push_str(&format!(
                "MINIMUM_BALANCE_RATIO={:.3}\n",
                *self.minimum_balance_ratio())
        );
        output.push_str(&format!(
                "MINIMUM_STAKE={:.2}\n",
                *self.minimum_stake() as f64 / 1_000_000.0)
        );
        output.push_str(&format!(
                "ADJUST_MINIMUM_STAKE={}\n",
                self.adjust_minimum_stake())
        );
        output.push_str(&format!(
                "MINIMUM_STAKE_ROUNDING={:.2}\n", 
                *self.minimum_stake_rounding() as f64 / 1_000_000.0)
        );
        output.push_str(&format!(
                "DAILY_REWARD={:.2}\n",
                *self.daily_reward() as f64 / 1_000_000.0)
        );

        // Add validators to the output
        for validator in self.config_validators()? {
            output.push_str(&format!(
                "read -r VALIDATOR MONIKER <<< \"{} {}\"\n",
                validator.address, validator.moniker
            ));
        }

        Ok(output)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P, overwrite: bool) -> eyre::Result<()> {
        let path = path.as_ref();

        if path.exists() && !overwrite {
            return Err(eyre::eyre!("File already exists at {:?} and overwrite is not allowed.", path));
        }

        let file = File::create(path).map_err(|e| eyre::eyre!("Failed to create file: {}", e))?;
        let mut writer = BufWriter::new(file);

        let config_str = self.export()?;
        writer.write_all(config_str.as_bytes())
            .with_context(|| format!("Failed to write to file: {:?}", path))?;

        Ok(())
    }

    pub fn add_validator(&mut self, search: &str) -> eyre::Result<&mut Self> {
        // Split the input by commas and trim any whitespace
        let parts: Vec<&str> = search.split(',').map(|s| s.trim()).collect();

        let mut config_validators = self.config_validators()?.clone();

        match parts.len() {
            // Case 1: Input is in the format "<address>,<moniker>"
            2 => {
                let address = parts[0].to_string();
                let moniker = parts[1].to_string();

                // Check if the address already exists in config_validators
                if let Some(validator) = config_validators.iter_mut().find(|v| v.address == address) {
                    // Update the moniker if the address exists
                    validator.moniker = moniker.clone();
                    println!("Validator with address {} updated with new moniker: {}", address, moniker);
                } else {
                    // If the address does not exist, append a new validator
                    config_validators.push(ConfigValidator::new(address.clone(), moniker.clone()));
                    println!("Added new validator with address: {} and moniker: {}", address, moniker);
                }
            },
            // Case 2: Input is a single search term (address or moniker)
            1 => {
                let search_term = parts[0];

                // Search the validators collection for the matching address or moniker
                let validators = self.validators()?;
                let found_validators = validators.search(search_term)?;

                // Loop through all found validators and add them if they don't exist in config_validators
                for validator in found_validators {
                    if config_validators.iter().any(|v| v.address == validator.address()) {
                        println!("Validator with address {} is already added.", validator.address());
                    } else {
                        config_validators.push(
                            ConfigValidator::new(
                                validator.address().to_string(), 
                                validator.moniker().to_string()
                            )
                        );
                        println!(
                            "Added validator with address: {} and moniker: {}", 
                            validator.address(), 
                            validator.moniker()
                        );
                    }
                }
            },
            // Case 3: Input has more than two parts, indicating multiple search terms
            _ if parts.len() > 2 => {
                // Convert parts into a vector of search strings
                let searches: Vec<String> = parts.iter().map(|s| s.to_string()).collect();

                // Use the search_multi method to find multiple validators
                let validators = self.validators()?;
                let found_validators = validators.search_multi(searches)?;

                // Add each validator found to config_validators if not already present
                for validator in found_validators {
                    if config_validators.iter().any(|v| v.address == validator.address()) {
                        println!(
                            "Validator with address {} is already added.",
                            validator.address()
                        );
                    } else {
                        config_validators.push(
                            ConfigValidator::new(
                                validator.address().to_string(),
                                validator.moniker().to_string(),
                            )
                        );
                        println!(
                            "Added validator with address: {} and moniker: {}", 
                            validator.address(), 
                            validator.moniker()
                        );
                    }
                }
            },
            _ => {
                // Invalid input format, return error
                return Err(eyre::eyre!(
                    "Invalid input. Expected format: '<address>,<moniker>' or '<search_term>'"
                ));
            }
        }

        // Update the internal state with the new list of validators.
        self.config_validators = OnceCell::from(config_validators);

        Ok(self)
    }
}


impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Call the export method and write its result to the Debug output
        match self.export() {
            Ok(exported_string) => f.write_str(&exported_string),
            Err(e) => f.write_str(&format!("Error exporting: {}", e)),
        }
    }
}
