
use crate::globals::{
    CLAIM_FEE, 
    NOMIC, 
    NOMIC_LEGACY_VERSION,
    STAKE_FEE
};
use chrono::{Utc, DateTime};
use clap::ValueEnum;
use crate::key::PrivKey;
use crate::nonce;
use crate::profiles::Balance;
use crate::profiles::Config;
use crate::profiles::Delegations;
use crate::validators::Validator;
use crate::validators::ValidatorCollection;
use eyre::eyre;
use eyre::Result;
use eyre::WrapErr;
use home::home_dir;
use once_cell::sync::OnceCell;
use serde_json::json;
use serde_json::Value;
use std::cmp::PartialEq;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

#[derive(Clone)]
pub struct Profile {
    home:                          PathBuf,
    key:                           OnceCell<PrivKey>,
    config:                        OnceCell<Config>,
    balances:                      OnceCell<Balance>,
    delegations:                   OnceCell<Delegations>,
    validators:                    OnceCell<ValidatorCollection>,
    validator:                     OnceCell<Validator>,
    name_result:                   OnceCell<String>,
}

impl Profile {
    pub fn new<P: AsRef<Path>>(home: Option<P>) -> Result<Self> {
        let home = home
            .map(|h| h.as_ref().to_path_buf())
            .or_else(home_dir)
            .ok_or_else(|| eyre!("Home directory must be provided or resolvable."))?;

        // Create the Profile instance with a temporary `None` for stats
        Ok(Profile {
            home,
            key:                           OnceCell::new(),
            config:                        OnceCell::new(),
            balances:                      OnceCell::new(),
            delegations:                   OnceCell::new(),
            validators:                    OnceCell::new(),
            validator:                     OnceCell::new(),
            name_result:                   OnceCell::new(),
        })
    }

    #[allow(dead_code)]
    pub fn init(self) -> Result<Self> {
        // Capture the current `home` value
        let home = self.home.clone();

        // Create a new Profile using the existing `home`
        Profile::new(Some(home))
    }

    /// Returns the validated home folder
    /// Checks if the folder exists, if not tries to create it
    /// if it does not exist and cannot create it, not much point of the profile
    /// if it does exist return it
    pub fn home_result(&self) -> eyre::Result<&Path> {
        // Check if folder exists
        if !self.home.exists() {
            // Attempt to create home folder
            fs::create_dir_all(&self.home)
                .wrap_err_with(|| format!("Failed to create directory: {:?}", &self.home))?;
        }
        Ok(&self.home)
    }

    /// home_str -> &str
    pub fn home_str(&self) -> &str {
        self.home_result()
            .map(|path| path.to_str().unwrap_or(""))  // Convert Path to &str, handle conversion failure with empty string
            .unwrap_or_else(|e| {
                eprintln!("{}", e);                   // Print the error context if any
                ""                                    // Return an empty string in case of an error
            })
    }

    /// Returns the wallet path
    /// We attemot to create the wallet path here if it does not exist
    /// not much point without a wallet
    pub fn wallet_path(&self) -> Result<PathBuf> {

        let wallet_path = self.home.join(".orga-wallet"); // Constructs the wallet path

        // Attempt to create the directory and all its parents if they don't exist
        fs::create_dir_all(&wallet_path)
            .wrap_err_with(|| format!("Failed to create directory: {:?}", wallet_path))?;

        Ok(wallet_path) // Returns the path
    }

    /// Returns the config file path.
    /// We just want the path here, we leave verification to the Config struct
    pub fn config_file(&self) -> Result<PathBuf> {
        Ok(self.home.join("config"))
    }

    /// Returns the key file path.
    /// We only need the path, verification left to the Key struct
    pub fn key_file(&self) -> Result<PathBuf> {
        let wallet_path = self.wallet_path()?;
        Ok(wallet_path.join("privkey"))
    }

    /// Returns the nonce file path.
    /// only need the path all other stuff by the Nonce module
    pub fn nonce_file(&self) -> Result<PathBuf> {
            Ok(self.wallet_path()?.join("nonce"))
    }

    /// Get the key, file read operation, 
    /// OnceCell used to cache the results
    pub fn key(&self) -> Result<&PrivKey> {
        self.key.get_or_try_init(|| {
            PrivKey::load(self.key_file()?, true)
        })
    }

    /// Get the address
    /// Operation by cosmrs cache with OnceCell
    /// cached in OnceCell as self.key.address() -> Result<&str>
    /// use self.key.address() to propagate errors
    /// this for display purposes
    pub fn address(&self) -> String {
        match self.key.get() {
            Some(key) => match key.address() {
                Ok(addr) => addr.to_string(), // Return the address as String
                Err(_) => "Failed to retrieve address".to_string(), // Handle errors
            },
            None => "Key is not initialized".to_string(), // If key is not present in OnceCell
        }
    }

    /// Get the export
    /// Operation by cosmrs cache with OnceCell
    /// cached in OnceCell as self.key.export() -> Result<&str>
    /// use self.key.export() to propagate errors
    /// this for display purposes
    pub fn export(&self) -> String {
        match self.key.get() {
            Some(key) => match key.export() {
                Ok(addr) => addr.to_string(),
                Err(_) => "Failed to retrieve export".to_string(),
            },
            None => "Key is not initialized".to_string(),
        }
    }

    /// Retrieves config, initializing it if necessary.
    /// file read operation, cache with OnceCell
    pub fn config(&self) -> Result<&Config> {
        self.config.get_or_try_init(|| {
            Config::load(self.config_file()?, true)
        })
    }

    /// Retrieves the balance, initializing it if necessary.
    /// blockchain operation, cache with oncecell
    pub fn balances(&self) -> eyre::Result<&Balance> {
        self.balances.get_or_try_init(|| {
            Balance::fetch(Some(self.key()?.address()?))
        })
    }

    /// Retrieves delegations, initializing it if necessary.
    /// blockchain operation, cache with oncecell
    pub fn delegations(&self) -> Result<&Delegations> {
        self.delegations.get_or_try_init(|| {
            Delegations::fetch(Some(self.home_result()?))
        })
    }

    /// Retrieves validators, initializing it if necessary.
    /// blockchain operation, cache with oncecell
    pub fn validators(&self) -> Result<&ValidatorCollection> {
        self.validators.get_or_try_init(|| {
            ValidatorCollection::fetch()
        })
    }

    /// gets the name of the profile from the path
    /// or default if the home is ystem home
    /// Complex logic, cache with OnnceCell
    pub fn name_result(&self) -> eyre::Result<&str> {
        // Attempt to get the name, initializing it if not already set
        self.name_result.get_or_try_init(|| {
            let home = self.home_result()?;
            // Attempt to get the default home directory
            match home_dir() {
                Some(default_home_dir) => {
                    // Check if the current home is the default home directory
                    if home == default_home_dir {
                        // Return a static string reference for the default case
                        Ok("default".to_string()) // Convert to String
                    } else {
                        // Extract the last component of the path and return it as a String
                        home.file_name()
                            .and_then(|name| name.to_str()) // This gives Option<&str>
                            .map(|s| s.to_string()) // Convert to String
                            .ok_or_else(|| eyre::eyre!("Failed to extract the last component of the path"))
                    }
                },
                None => Err(eyre::eyre!("Failed to get the default home directory")),
            }
        })
        .map(|s| s.as_str()) // Convert String back to &str
    }

    /// name -> &str
    pub fn name(&self) -> &str {
        self.name_result()
        .unwrap_or_else(|e| {
            eprintln!("{}", e); // Print the error context if any
            ""                  // Return an empty string in case of an error
        })
    }

    /// Setter for the validators field with more flexible error handling.
    /// Allow cached validators from a calling function or struct
    pub fn set_validators(&mut self, validators: ValidatorCollection) -> Result<()> {
        // Try to set the validators only if it hasn't been set before.
        self.validators
            .set(validators)
            .map_err(|_| eyre!("Validators have already been set"))
    }

    /// import a new private key into profile
    pub fn import(&self, hex_str: &str, force: bool) -> Result<()> {
        let key_file = self.key_file()?; // Get the key file path

        // Check if the key file already exists
        if key_file.exists() && !force {
            return Err(eyre::eyre!("Key file already exists. Use 'force' to overwrite it."));
        }

        // Import the private key from the hex string
        let key = PrivKey::import(hex_str)?;

        // Save the key to the key file
        key.save(key_file, force)?;

        Ok(())
    }

    pub fn export_nonce(&self) -> Result<u64> {
        let nonce_file = self.nonce_file()?;
        nonce::export(Some(&nonce_file), None)
    }

    pub fn import_nonce(&self, value: u64, dont_overwrite: bool) -> Result<()> {
        let nonce_file = self.nonce_file()?;
        nonce::import(value, Some(&nonce_file), None, dont_overwrite)
    }

    /// balance -> u64
    /// self.balances()?.nom ->Result<u64>
    pub fn balance(&self) -> u64 {
        self.balances()               // get delegations
        .map(|bal| bal.nom)           // if Ok get the timestamp
        .unwrap_or_else(|e| {
            eprintln!("{}", e);       // could not get balances error to stderr
            0                         // default value
        })
    }

    /// timestamp -> DateTime<Utc>
    /// self.delegations()?.timestamp -> Result<DateTime<Utc>>
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.delegations()             // get delegations
        .map(|del| del.timestamp)      // if Ok get the timestamp
        .unwrap_or_else(|e| {
            eprintln!("{}", e);        // could not get delegations error to stderr
            Utc::now()                 // default value
        })
    }

    /// total_staked -> u64
    /// self.delegations()?.total().staked -> Result<u64>
    pub fn total_staked(&self) -> u64 {
        self.delegations()             // get delegations
        .map(|del| del.total().staked) // if Ok get the total staked
        .unwrap_or_else(|e| {
            eprintln!("{}", e);        // could not get delegations error to stderr
            0                          // default value
        })
    }

    /// total_liquid -> u64
    /// self.delegations()?.total().liquid -> Result<u64>
    pub fn total_liquid(&self) -> u64 {
        self.delegations()              // get delegations
        .map(|del| del.total().liquid)  // if Ok get the total staked
        .unwrap_or_else(|e| {
            eprintln!("{}", e);         // could not get  delegations error to stderr
            0                           // default value
        })
    }

    /// config_minimum_balance -> u64
    /// self.config()?.minimum_balance -> Result<u64>
    pub fn config_minimum_balance(&self) -> u64 {
        self.config()
        .map(|config| config.minimum_balance)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            10_000
        })
    }

    /// config_minimum_balance_ratio -> f64
    /// self.config()?.minimum_balance_ratio -> Result<f64>
    pub fn config_minimum_balance_ratio(&self) -> f64 {
        self.config()
        .map(|config| config.minimum_balance_ratio)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            0.001
        })
    }

    /// config_minimum_stake -> u64
    /// self.config()?.minimum_stake -> Result<u64>
    pub fn config_minimum_stake(&self) -> u64 {
        self.config()
        .map(|config| config.minimum_stake)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            1_000_000
        })
    }

    /// config_adjust_minimum_stake -> bool
    /// self.config()?.adjust_minimum_stake -> Result<bool>
    pub fn config_adjust_minimum_stake(&self) -> bool {
        self.config()
        .map(|config| config.adjust_minimum_stake)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            false
        })
    }

    /// config_minimum_stake_rounding -> u64
    /// self.config()?.minimum_stake_rounding -> Result<u64>
    pub fn config_minimum_stake_rounding(&self) -> u64 {
        self.config()
        .map(|config| config.minimum_stake_rounding)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            10_000
        })
    }

    /// config_daily_reward -> f64
    /// self.config()?.daily_reward -> Result<f64>
    pub fn config_daily_reward(&self) -> f64 {
        self.config()
        .map(|config| config.daily_reward)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            0.0
        })
    }

    /// config_validator -> &str
    /// self.config()?.active_validator()?.address -> Result<&str>
    pub fn config_validator(&self) -> &str {
        self.config()
            .and_then(|config| config.active_validator())  // Access the active validator
            .map(|validator| validator.address.as_str())   // Get the address as &str
            .unwrap_or_else(|e| {
                eprintln!("{}", e);                        // Print the error context if any
                ""                                         // Return an empty string in case of an error
            })
    }

    /// config_moniker -> &str
    /// self.config()?.active_validator()?.moniker -> Result<&str>
    pub fn config_moniker(&self) -> &str {
        self.config()
            .and_then(|config| config.active_validator())  // Access the active validator
            .map(|validator| validator.moniker.as_str())   // Get the moniker as &str
            .unwrap_or_else(|e| {
                eprintln!("{}", e);                        // Print the error context if any
                ""                                         // Return an empty string in case of an error
            })
    }

    /// self.validators()?.validator(self.config_validator())? -> Result<Validator>
    pub fn validator(&self) -> eyre::Result<Validator> {
        self.validator.get_or_try_init(|| {
            Ok(self.validators()?.validator(self.config_validator())?.clone())
        }).cloned()
    }

    /// Retrieves the moniker of the validator as a String.
    pub fn moniker(&self) -> String {
        match self.validator() {
            Ok(validator) => validator.moniker().to_string(), // Return the moniker as a borrowed reference
            Err(e) => {
                eprintln!("{}", e); // Print the error context if any
                "".to_string() // Return an empty string in case of an error
            }
        }
    }

    /// Retrieves the voting_power of the validator as a String.
    pub fn voting_power(&self) -> u64 {
        match self.validator() {
            Ok(validator) => validator.voting_power(), // Return the voting_power as a borrowed reference
            Err(e) => {
                eprintln!("{}", e); // Print the error context if any
                0 // Return an empty string in case of an error
            }
        }
    }

    /// rank -> u64
    pub fn rank(&self) -> u64 {
        match self.validator() {
            Ok(validator) => validator.rank(), // Return the rank as a borrowed reference
            Err(e) => {
                eprintln!("{}", e); // Print the error context if any
                0 // Return an empty string in case of an error
            }
        }
    }

    /// validator_staked -> u64
    /// self.delegations.delegations.get(self.config_validator())?.staked -> Result<u64>
    pub fn validator_staked(&self) -> u64 {
        self.delegations()
            .and_then(|del| {
                // Get the delegation for the active validator and access the staked value
                del.delegations
                    .get(self.config_validator())
                    .map(|delegation| delegation.staked) // Map to the staked value
                    .ok_or_else(|| eyre::eyre!("No delegation found for the validator")) // Wrap in Ok
            })
            .unwrap_or_else(|e| {
                eprintln!("{}", e); // Print the error context if any
                0 // Return a default value in case of an error
            })
    }

    pub fn claim_fee(&self) -> u64 {
        (*CLAIM_FEE * 1_000_000.0) as u64
    }

    pub fn stake_fee(&self) -> u64 {
        (*STAKE_FEE * 1_000_000.0) as u64
    }

    pub fn minimum_balance(&self) -> u64 {
        // Calculate the minimum required balance based on the ratio
        let calculated_min = (self.total_staked() as f64 * self.config_minimum_balance_ratio()).ceil() as u64;

        // Return the maximum of the calculated minimum and configured minimum
        self.config_minimum_balance().max(calculated_min)

    }

    pub fn available_before_claim(&self) -> u64 {
        // Calculate available amount before claim
        self.balance()
            .saturating_sub(self.minimum_balance())
            .saturating_sub(self.stake_fee())
            .max(0)
    }

    pub fn available_after_claim(&self) -> u64 {
        // Calculate available amount after the claim
        self.balance()
            .saturating_add(self.total_liquid())
            .saturating_sub(self.minimum_balance())
            .saturating_sub(self.claim_fee())
            .saturating_sub(self.stake_fee())
            .max(0)
    }

    pub fn stake_factor(&self) -> u64 {
        // Calculate the stake factor based on the rounding logic
        let min_stake = self.config_minimum_stake();
        let min_stake_rounding = self.config_minimum_stake_rounding();

        if min_stake < min_stake_rounding || min_stake_rounding == 0 {
            min_stake
        } else {
            min_stake
                .saturating_div(min_stake_rounding)
                .saturating_mul(min_stake_rounding)
        }
    }

    pub fn validator_staked_remainder(&self) -> u64 {
        self.validator_staked() % self.stake_factor()
    }

    pub fn can_stake_before_claim(&self) -> bool {
        let factor    = self.stake_factor();
        let available = self.available_before_claim();
        let remainder = self.validator_staked_remainder();

        // Determine if staking can occur without needing to claim
        if remainder > 0 {
            available > remainder
        } else {
            available > factor
        }
    }

    pub fn can_stake_after_claim(&self) -> bool {
        let factor    = self.stake_factor();
        let available = self.available_after_claim();
        let remainder = self.validator_staked_remainder();

        // Determine if staking can occur after claiming rewards
        if remainder > 0 {
            available > remainder
        } else {
            available > factor
        }

    }

    pub fn claim(&self) -> bool {
        !self.can_stake_before_claim() && self.can_stake_after_claim()
    }

    pub fn quantity_to_stake(&self) -> u64 {
        let can_stake_before_claim     = self.can_stake_before_claim();
        let can_stake_after_claim      = self.can_stake_after_claim();
        let available_before_claim     = self.available_before_claim();
        let available_after_claim      = self.available_after_claim();
        let stake_factor               = self.stake_factor();
        let validator_staked_remainder = self.validator_staked_remainder();

        // If staking is not possible either before or after claiming, return 0
        if !can_stake_before_claim && !can_stake_after_claim {
            return 0;
        }

        // Determine which available amount to use based on staking conditions
        let available = if can_stake_before_claim {
            available_before_claim
        } else {
            available_after_claim
        };

        // Calculate the quantity to stake
        let mut quantity_to_stake = validator_staked_remainder;

        // Calculate the leftover available amount after accounting for the remainder
        let leftover = available.saturating_sub(validator_staked_remainder);

        // Add multiples of `stake_factor` from the leftover
        quantity_to_stake += (leftover / stake_factor) * stake_factor;

        quantity_to_stake
    }
}

// Custom Debug implementation for Profile
impl std::fmt::Debug for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Profile {{ address: {}, name: {} }}",
            self.address(), self.name()
        )
    }
}

impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        // Compare the addresses of the two profiles directly
        self.address() == other.address()
    }
}

impl Profile {

    pub fn nomic_claim(&self) -> eyre::Result<()> {

        // Create and configure the Command for running "nomic claim"
        let mut cmd = Command::new(&*NOMIC);
        cmd.arg("claim");

        // Set the environment variables for NOMIC_LEGACY_VERSION
        cmd.env("NOMIC_LEGACY_VERSION", &*NOMIC_LEGACY_VERSION);

        // Set the HOME environment variable
        cmd.env("HOME", self.home_str().to_string());

        // Execute the command and collect the output
        let output = cmd.output()?;

        // Check if the command was successful
        if !output.status.success() {
            let error_msg = format!(
                "Command `{}` failed with output: {:?}",
                &*NOMIC,
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(eyre!(error_msg));
        }

        Ok(())
    }

    pub fn nomic_delegate(&self) -> eyre::Result<()> {


        let validator = self.config_validator();
        let quantity = self.quantity_to_stake();

        // Create and configure the Command for running "nomic delegate"
        let mut cmd = Command::new(&*NOMIC);

        // Set the environment variables for NOMIC_LEGACY_VERSION
        cmd.env("NOMIC_LEGACY_VERSION", &*NOMIC_LEGACY_VERSION);

        // Set the HOME environment variable
        cmd.env("HOME", self.home_str().to_string());

        // Add the "delegate" argument, validator, and quantity
        cmd.arg("delegate");
        cmd.arg(validator);
        cmd.arg(quantity.to_string());

        // Execute the command and collect the output
        let output = cmd.output()?;

        // Check if the command was successful
        if !output.status.success() {
            let error_msg = format!(
                "Command `{}` failed with output: {:?}",
                &*NOMIC,
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(eyre!(error_msg));
        }

        // Clone the config
        let mut config = self.config()?.clone();

        // Rotate the config validators
        config.rotate_validators();

        // Save config to disk
        config.save(self.config_file()?, true)?;
        Ok(())

    }

    pub fn auto_delegate(&self, log: bool) -> eyre::Result<()> {
        let quantity = self.quantity_to_stake();
        let claim = self.claim();

        // Log the current state if requested
        if log {
            self.print(Some(OutputFormat::Json))?;
        }

        if quantity > 0 {
            if claim {
                self.nomic_claim()?;
            }
            self.nomic_delegate()?;
        } else {
            // Log a message since this is not an error
            eprintln!("Not enough to stake");
        }
        Ok(())
    }

}

impl Profile {
    pub fn json(&self) -> eyre::Result<Value> {
        // Call all methods and collect their results
        let json_output = json!({
            "profile":                       self.name(),
            "address":                       self.address(),
            "balance":                       self.balance(),
            "total_staked":                  self.total_staked(),
            "timestamp":                     self.timestamp().to_rfc3339(),
            "total_liquid":                  self.total_liquid(),
            "config_minimum_balance":        self.config_minimum_balance(),
            "config_minimum_balance_ratio":  self.config_minimum_balance_ratio(),
            "config_minimum_stake":          self.config_minimum_stake(),
            "config_adjust_minimum_stake":   self.config_adjust_minimum_stake(),
            "config_minimum_stake_rounding": self.config_minimum_stake_rounding(),
            "config_daily_reward":           self.config_daily_reward(),
            "config_validator":              self.config_validator(),
            "config_moniker":                self.config_moniker(),
            "moniker":                       self.moniker(),
            "voting_power":                  self.voting_power(),
            "rank":                          self.rank(),
            "validator_staked":              self.validator_staked(),
            "claim_fee":                     self.claim_fee(),
            "stake_fee":                     self.stake_fee(),
            "minimum_balance":               self.minimum_balance(),
            "stake_factor":                  self.stake_factor(),
            "available_before_claim":        self.available_before_claim(),
            "available_after_claim":         self.available_after_claim(),
            "validator_staked_remainder":    self.validator_staked_remainder(),
            "can_stake_before_claim":        self.can_stake_before_claim(),
            "can_stake_after_claim":         self.can_stake_after_claim(),
            "daily_reward":                  self.config_daily_reward(),
            "claim":                         self.claim(),
            "quantity_to_stake":             self.quantity_to_stake(),
        });

        Ok(json_output)
    }
}

/// Enum to represent output formats
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    JsonPretty,
}

impl FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json"        => Ok(OutputFormat::Json),
            "json-pretty" => Ok(OutputFormat::JsonPretty),
            _             => Err(format!("Invalid output format: {}", s)),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            OutputFormat::Json       => "json",
            OutputFormat::JsonPretty => "json-pretty",
        };
        write!(f, "{}", output)
    }
}

impl Profile {
    pub fn print(&self,
        format: Option<OutputFormat>,
    ) -> eyre::Result<()> {

        // Use the default format if None is provided
        let format = format.unwrap_or(OutputFormat::JsonPretty);

        match format {
            OutputFormat::Json => {
                let json_value = self.json()?;
                let json_str = serde_json::to_string(&json_value)
                    .map_err(|e| eyre::eyre!("Error serializing JSON: {}", e))?;
                println!("{}", json_str);
            },
            OutputFormat::JsonPretty => {
                let json_value = self.json()?;
                let pretty_json = serde_json::to_string_pretty(&json_value)
                    .map_err(|e| eyre::eyre!("Error serializing JSON: {}", e))?;
                println!("{}", pretty_json);
            },
        }

        Ok(())
    }


    pub fn edit_config(
        &self,
        minimum_balance: Option<u64>,
        minimum_balance_ratio: Option<f64>,
        minimum_stake: Option<u64>,
        adjust_minimum_stake: Option<bool>,
        minimum_stake_rounding: Option<u64>,
    ) -> eyre::Result<()> {
        // Check if all inputs are None, return an error
        if minimum_balance.is_none()
            && minimum_balance_ratio.is_none()
            && minimum_stake.is_none()
            && adjust_minimum_stake.is_none()
            && minimum_stake_rounding.is_none()
        {
            return Err(eyre!("At least one input must be provided to edit the config."));
        }

        // Clone the config to modify it
        let mut config = self.config()?.clone();

        // Apply changes only if the corresponding option is provided
        if let Some(balance) = minimum_balance {
            config.minimum_balance = balance;
        }
        if let Some(balance_ratio) = minimum_balance_ratio {
            config.minimum_balance_ratio = balance_ratio;
        }
        if let Some(stake) = minimum_stake {
            config.minimum_stake = stake;
        }
        if let Some(adjust_stake) = adjust_minimum_stake {
            config.adjust_minimum_stake = adjust_stake;
        }
        if let Some(stake_rounding) = minimum_stake_rounding {
            config.minimum_stake_rounding = stake_rounding;
        }

        // Save the updated config if needed
        config.save(self.config_file()?, true)?;

        Ok(())
    }
}
