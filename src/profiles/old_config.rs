use crate::functions::to_bool;
use crate::globals::{
    ADJUST_MINIMUM_STAKE, 
    MINIMUM_BALANCE, 
    MINIMUM_BALANCE_RATIO, 
    MINIMUM_STAKE, 
    MINIMUM_STAKE_ROUNDING, 
};
use crate::validators::ValidatorCollection;
use eyre::ContextCompat;
use eyre::WrapErr;
use eyre::Result;
use once_cell::sync::OnceCell;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use chrono::{Utc, DateTime};
use std::cell::Ref;
use std::cell::RefCell;


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

pub struct Config {
    profile:                           Option<String>,
    content:                                   String,
    minimum_balance:                    OnceCell<u64>,
    minimum_balance_ratio:              OnceCell<f64>,
    minimum_stake:                      OnceCell<u64>,
    adjust_minimum_stake:              OnceCell<bool>,
    minimum_stake_rounding:             OnceCell<u64>,
    daily_reward:                       OnceCell<f64>,
    validator:                       OnceCell<String>,
    moniker:                         OnceCell<String>,
    active_validator:       OnceCell<ConfigValidator>,
    config_validators:   Option<Vec<ConfigValidator>>,
    validators:         OnceCell<ValidatorCollection>,
    #[allow(dead_code)]
    timestamp:                          DateTime<Utc>,
}

fn capture_value(variable_name: &str, content: &str) -> Option<String> {
    // Construct the regex pattern dynamically using the variable name
    let pattern = format!(
        r"(?m)^[[:space:]]*{var_name}(?:[[:space:]]*=[[:space:]]*|[[:space:]]+)*([0-9]+|[0-9]*\.[0-9]+).*$",
        var_name = variable_name
    );

    let regex = Regex::new(&pattern).unwrap();

    // Attempt to capture the value
    if let Some(captures) = regex.captures(content) {
        if let Some(value) = captures.get(1) { // Get the first capturing group (the value)
            return Some(value.as_str().to_string()); // Return the captured value as String
        }
    }

    // Return None if no match found
    None
}

impl Config {

    pub fn new<P: AsRef<Path>>(
        profile: Option<String>,
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

        // Initialize OnceCell<ValidatorCollection>
        let validators = validators
            .map(|v| {
                let cell = OnceCell::new();
                cell.set(v).unwrap(); // Ensure OnceCell is initialized with the provided collection
                cell
            })
            .unwrap_or_else(OnceCell::new); // Initialize as an empty OnceCell if None

        Ok(Self {
            profile,
            content,
            minimum_balance:        OnceCell::new(),
            minimum_balance_ratio:  OnceCell::new(),
            minimum_stake:          OnceCell::new(),
            adjust_minimum_stake:   OnceCell::new(),
            minimum_stake_rounding: OnceCell::new(),
            daily_reward:           OnceCell::new(),
            validator:              OnceCell::new(),
            moniker:                OnceCell::new(),
            active_validator:       OnceCell::new(),
            config_validators:      OnceCell::new(),
            validators,
            timestamp,
        })
    }

    pub fn minimum_balance(&self) -> Result<&u64> {
        self.minimum_balance.get_or_try_init(|| {
            // Check the evironment variable
            if let Some(balance) = *MINIMUM_BALANCE {
                return Ok((balance * 1_000_000.0) as u64);
            }

            // Check the configuration file
            if let Some(matched_value) = capture_value("MINIMUM_BALANCE", &self.content) {
                // Multiply by 1,000,000 before converting to u64
                if let Ok(balance) = matched_value.parse::<f64>() {
                    return Ok((balance * 1_000_000.0) as u64);
                }
            }

            // Default value if none found
            Ok(100_000)
        })
    }

    pub fn minimum_balance_ratio(&self) -> Result<&f64> {
        self.minimum_balance_ratio.get_or_try_init(|| {
            // Check the evironment variable
            if let Some(ratio) = *MINIMUM_BALANCE_RATIO {
                return Ok(ratio);
            }

            // Check the configuration file
            if let Some(matched_value) = capture_value("MINIMUM_BALANCE_RATIO", &self.content) {
                if let Ok(ratio) = matched_value.parse::<f64>() {
                    return Ok(ratio);
                }
            }

            // Default value if none found
            Ok(0.001) // Default ratio value
        })
    }

    pub fn minimum_stake(&self) -> Result<&u64> {
        self.minimum_stake.get_or_try_init(|| {
            // Check the evironment variable
            if let Some(stake) = &*MINIMUM_STAKE {
                return Ok((stake * 1_000_000.0) as u64);
            }

            // Check the configuration file
            if let Some(matched_value) = capture_value("MINIMUM_STAKE", &self.content) {
                if let Ok(stake) = matched_value.parse::<f64>() {
                    return Ok((stake * 1_000_000.0) as u64);
                }
            }

            // Default value if none found
            Ok(1_000_000)
        })
    }

    // pub fn to_bool_string(val: String) -> Option<String> {
    pub fn adjust_minimum_stake(&self) -> Result<&bool> {
        self.adjust_minimum_stake.get_or_try_init(|| {
            // Check the environment variable
            if let Some(adjust) = &*ADJUST_MINIMUM_STAKE {
                // Convert the value to a bool using to_bool function
                if let Some(min) = to_bool(adjust.to_string()) {
                    return Ok(min);
                }
            }

            // Check the configuration file
            if let Some(adjust) = capture_value("ADJUST_MINIMUM_STAKE", &self.content) {
                // Convert the value to a bool using to_bool function
                if let Some(min) = to_bool(adjust) {
                    return Ok(min);
                }
            }

            // Default value if none found
            Ok(false)
        })
    }

    pub fn minimum_stake_rounding(&self) -> Result<&u64> {
        self.minimum_stake_rounding.get_or_try_init(|| {
            // Check the environment variable
            if let Some(round) = *MINIMUM_STAKE_ROUNDING {
                // Multiply by 1,000,000 and convert to u64
                return Ok((round * 1_000_000.0) as u64);
            }

            // Check the configuration file
            if let Some(matched_value) = capture_value("MINIMUM_STAKE_ROUNDING", &self.content) {
                // Multiply by 1,000,000 before converting to u64
                if let Ok(round) = matched_value.parse::<f64>() {
                    return Ok((round * 1_000_000.0) as u64);
                }
            }

            // Default value if none found
            Ok(10_000)
        })
    }

    pub fn daily_reward(&self) -> Result<&f64> {
        self.daily_reward.get_or_try_init(|| {

            // Check the configuration file
            if let Some(matched_value) = capture_value("DAILY_REWARD", &self.content) {
                if let Ok(reward) = matched_value.parse::<f64>() {
                    return Ok(reward);
                }
            }

            // Default value if none found
            Ok(0.0) // Default reward value
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
    pub fn config_validators(&mut self) -> eyre::Result<&Vec<ConfigValidator>> {
        if self.config_validators.is_none() {
            let mut config_validators = Vec::new();

            let comment_regex = Regex::new(r"^\s*#").unwrap();
            let read_validator_regex = Regex::new(concat!(
                r"^\s*read\s+(?:-r\s+)?VALIDATOR\s+MONIKER\s+<<<\s*",
                r#"["]?([^\s"']+)["]?\s+["]?([^"\s]+)["]?"#
            )).unwrap();

            // Iterate over the lines of the content
            for line in self.content.lines() {
                let line = line.trim();

                // Skip empty lines and comments
                if line.is_empty() || comment_regex.is_match(line) {
                    continue;
                }

                // Capture the address and moniker if the regex matches
                if let Some(captures) = read_validator_regex.captures(line) {
                    let address = captures.get(1)
                        .context("Missing validator address")?
                        .as_str()
                        .to_string();
                    let moniker = captures.get(2)
                        .context("Missing validator moniker")?
                        .as_str()
                        .to_string();

                    config_validators.push(ConfigValidator { address, moniker });
                }
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

            // Set the config_validators after initialization
            self.config_validators = Some(config_validators);
        }

        // Return a reference to the validators
        self.config_validators.as_ref().context("Config validators are missing")
    }


    pub fn active_validator(&self) -> eyre::Result<&ConfigValidator> {
        // Get the validators, ensuring it's initialized
        let validators = self.config_validators()?;

        // Return the last validator
        validators.last() // Get the last item from the vector
            .ok_or_else(|| eyre::eyre!("No validators found"))
    }

    pub fn validator(&mut self) -> eyre::Result<&str> {
            Ok(&self.active_validator()?.address)
    }

    pub fn moniker(&mut self) -> eyre::Result<&str> {
            Ok(&self.active_validator()?.moniker)
    }

    pub fn remove_validator(&mut self, search: &str) -> eyre::Result<&mut Self> {
        // Clone the existing validators and filter them
        let mut validators = self.config_validators()?.clone();

        // Retain only the validators that don't match the search criteria
        validators.retain(|validator| {
            !(validator.address == search || validator.moniker == search)
        });

        // Create a new OnceCell and replace the old one
        let new_validators = OnceCell::new();
        new_validators.set(validators).map_err(|_| eyre::eyre!("Unable to update validators"))?;

        // Replace the old OnceCell with the new one
        self.config_validators = new_validators;

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

        // Create a new OnceCell and try to set the new list of validators
        let new_validators = OnceCell::new();
        new_validators
            .set(validators)
            .map_err(|_| eyre::eyre!("Unable to update validators"))?;

        // Replace the old OnceCell with the new one
        self.config_validators = new_validators;

        Ok(self)
    }

    pub fn export(&self) -> eyre::Result<String> {
        let mut output = String::new();

        // Add profile information, handle potential None value
        output.push_str(&format!("PROFILE={}\n",self.profile.as_deref().unwrap_or("unspecified")));

        // Add financial parameters
        output.push_str(&format!("MINIMUM_BALANCE={:.2}\n", *self.minimum_balance()? as f64 / 1_000_000.0));
        output.push_str(&format!("MINIMUM_BALANCE_RATIO={:.3}\n", *self.minimum_balance_ratio()?));
        output.push_str(&format!("MINIMUM_STAKE={:.2}\n", *self.minimum_stake()? as f64 / 1_000_000.0));
        output.push_str(&format!("ADJUST_MINIMUM_STAKE={}\n", self.adjust_minimum_stake()?));
        output.push_str(&format!("MINIMUM_STAKE_ROUNDING={:.2}\n", *self.minimum_stake_rounding()? as f64 / 1_000_000.0));
        output.push_str(&format!("DAILY_REWARD={:.2}\n", *self.daily_reward()? as f64 / 1_000_000.0));

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

    // Access or initialize config_validators. Since we cannot reset the OnceCell, we access the vector directly.
    let mut config_validators = self.config_validators.get_or_init(|| Vec::new());

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
                    config_validators.push(ConfigValidator::new(validator.address().to_string(), validator.moniker().to_string()));
                    println!("Added validator with address: {} and moniker: {}", validator.address(), validator.moniker());
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
                    println!("Validator with address {} is already added.", validator.address());
                } else {
                    config_validators.push(ConfigValidator::new(validator.address().to_string(), validator.moniker().to_string()));
                    println!("Added validator with address: {} and moniker: {}", validator.address(), validator.moniker());
                }
            }
        },
        _ => {
            // Invalid input format, return error
            return Err(eyre::eyre!("Invalid input. Expected format: '<address>,<moniker>' or '<search_term>'"));
        }
    }

    Ok(self)
}



}




//  pub fn new2(
//      config_str: String, 
//      validators: Option<ValidatorCollection>,
//      timestamp:  Option<DateTime<Utc>>,
//  ) -> eyre::Result<Self> {
//      let mut config_map = HashMap::new();
//      let mut config_validators = Vec::new();
//      let comment_regex = Regex::new(r"^\s*#").unwrap();
//      let read_validator_regex = Regex::new(concat!(
//          r"^\s*read\s+(?:-r\s+)?VALIDATOR\s+MONIKER\s+<<<\s*",
//          r#"["]?([^\s"']+)["]?\s+["]?([^"\s]+)["]?"#
//      )).unwrap();

//      for line in config_str.lines() {
//          let line = line.trim();

//          if line.is_empty() || comment_regex.is_match(line) {
//              continue;
//          }

//          if let Some((key, value)) = line.split_once('=') {
//              config_map.insert(key.trim().to_string(), value.trim().to_string()); // Convert to owned String
//          } else if let Some(captures) = read_validator_regex.captures(line) {
//              let address = captures.get(1).context("Missing validator address")?.as_str().to_string(); // Convert to owned String
//              let moniker = captures.get(2).context("Missing validator moniker")?.as_str().to_string(); // Convert to owned String
//              config_validators.push(ConfigValidator { address, moniker });
//          }
//      }

//      let get_required_u64 = |key: &str, factor: f64| -> eyre::Result<u64> {
//          let value = config_map.get(key)
//              .context(format!("Missing {}", key))?
//              .parse::<f64>()
//              .context(format!("Invalid {}", key))?;

//          // Multiply and convert to u64
//          let result = (value * factor) as u64;
//          Ok(result)
//      };

//      let get_required_f64 = |key: &str, factor: f64| -> eyre::Result<f64> {
//          let value = config_map.get(key)
//              .context(format!("Missing {}", key))?
//              .parse::<f64>()
//              .context(format!("Invalid {}", key))?;

//          // Multiply and convert to f64
//          let result = (value * factor) as f64;
//          Ok(result)
//      };

//      let minimum_balance = match *MINIMUM_BALANCE {
//          Some(val) => Ok((val * 1_000_000.0) as u64),
//          None => get_required_u64("MINIMUM_BALANCE", 1_000_000.0),
//      }?; // Use ? to propagate errors

//      let minimum_balance_ratio = match *MINIMUM_BALANCE_RATIO {
//          Some(val) => Ok(val),
//          None => get_required_f64("MINIMUM_BALANCE_RATIO", 1.0),
//      }?; // Use ? to propagate errors

//      let minimum_stake = match *MINIMUM_STAKE {
//          Some(val) => Ok((val * 1_000_000.0) as u64),
//          None => get_required_u64("MINIMUM_STAKE", 1_000_000.0),
//      }?; // Use ? to propagate errors

//      let minimum_stake_rounding = match *MINIMUM_STAKE_ROUNDING {
//          Some(val) => Ok((val * 1_000_000.0) as u64),
//          None => get_required_u64("MINIMUM_STAKE_ROUNDING", 1_000_000.0),
//      }?; // Use ? to propagate errors

//      let adjust_minimum_stake = match *ADJUST_MINIMUM_STAKE {
//          Some(ref val) => to_bool(val.clone()).unwrap_or(false),
//          None => config_map.get("ADJUST_MINIMUM_STAKE")
//              .context("Missing adjust minimum stake")? // Handle missing key
//              .parse::<bool>() // Attempt to parse to bool
//              .context("Invalid adjust minimum stake")?, // Handle parsing errors
//      };

//      // Correctly initialize OnceCell<ValidatorCollection>
//      let validators = match validators {
//          Some(v) => {
//              let cell = OnceCell::new();
//              cell.set(v).unwrap(); // Ensure OnceCell is initialized with the provided collection
//              cell
//          },
//          None => OnceCell::new(), // Initialize it as an empty OnceCell
//      };

//      // Set the timestamp, defaulting to the current time if none is provided
//      let timestamp = timestamp.unwrap_or_else(Utc::now);

//      Ok(Config {
//          profile: config_map.get("PROFILE").context("Missing profile")?.to_string(),
//          minimum_balance,
//          minimum_balance_ratio,
//          minimum_stake,
//          adjust_minimum_stake,
//          minimum_stake_rounding,
//          daily_reward: get_required_f64("DAILY_REWARD", 1_000_000.0)?,
//          config_validators,
//          validators,
//          timestamp,
//      })
//  }







//}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Call the export method and write its result to the Debug output
        match self.export() {
            Ok(exported_string) => f.write_str(&exported_string),
            Err(e) => f.write_str(&format!("Error exporting: {}", e)),
        }
    }
}
