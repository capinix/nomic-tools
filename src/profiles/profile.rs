
use crate::globals::{
    CLAIM_FEE, 
    NOMIC, 
    NOMIC_LEGACY_VERSION,
    STAKE_FEE,
};
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use crate::functions::prompt_user;
use crate::functions::is_valid_nomic_address;
use crate::functions::TaskStatus;
use crate::functions::json_table;
use crate::globals::PROFILES_DIR;
use crate::privkey::PrivKey;
use crate::nonce;
use crate::profiles::Balance;
use crate::profiles::Config;
use crate::profiles::Delegations;
use crate::profiles::ProfileCollection;
use crate::validators::Validator;
use crate::validators::ValidatorCollection;
use crate::validators::initialize_validators;
use eyre::eyre;
use eyre::Result;
use eyre::WrapErr;
use once_cell::sync::OnceCell;
use serde_json::json;
use serde_json::Value;
use std::cmp::PartialEq;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::env;


#[derive(Clone)]
pub struct Profile {
    #[allow(dead_code)]
    timestamp:                     DateTime<Utc>,
    name:                          String,
    home:                          PathBuf,
    key:                           OnceCell<PrivKey>,
    config:                        OnceCell<Config>,
    balances:                      OnceCell<Balance>,
    delegations:                   OnceCell<Delegations>,
    validators:                    OnceCell<ValidatorCollection>,
    validator:                     OnceCell<Validator>,
    config_file:                   OnceCell<PathBuf>,
    wallet_path:                   OnceCell<PathBuf>,
    nonce_file:                    OnceCell<PathBuf>,
    key_file:                      OnceCell<PathBuf>,
    address:                       OnceCell<String>,
    export:                        OnceCell<String>,
    balance:                       OnceCell<u64>,
    delegations_timestamp:         OnceCell<DateTime<Utc>>,
    total_staked:                  OnceCell<u64>,
    total_liquid:                  OnceCell<u64>,
    config_minimum_balance:        OnceCell<u64>,
    config_minimum_balance_ratio:  OnceCell<f64>,
    config_minimum_stake:          OnceCell<u64>,
    config_adjust_minimum_stake:   OnceCell<bool>,
    config_minimum_stake_rounding: OnceCell<u64>,
    config_daily_reward:           OnceCell<f64>,
    config_validator_address:      OnceCell<String>,
    config_validator_moniker:      OnceCell<String>,
    moniker:                       OnceCell<String>,
    voting_power:                  OnceCell<u64>,
    rank:                          OnceCell<u64>,
    validator_staked:              OnceCell<u64>,
    minimum_balance:               OnceCell<u64>,
    available_without_claim:       OnceCell<u64>,
    available_after_claim:         OnceCell<u64>,
    stake_factor:                  OnceCell<u64>,
    validator_staked_remainder:    OnceCell<u64>,
    can_stake_without_claim:       OnceCell<bool>,
    can_stake_after_claim:         OnceCell<bool>,
    needs_claim:                   OnceCell<bool>,
    quantity_to_stake:             OnceCell<u64>,
    daily_reward:                  OnceCell<f64>,
    last_journal:                  OnceCell<serde_json::Value>,
    staked:                        bool,
    claimed:                       bool,
}

// Static zero for use when there is an error
static ZERO: u64 = 0;

impl Profile {

    fn create_profile(
        timestamp:  DateTime<Utc>,
        name:       String,
        home:       PathBuf,
        validators: Option<ValidatorCollection>,
    ) -> Self {
        Self {
            timestamp,
            name,
            home,
            validators:                    initialize_validators(validators),
            key:                           OnceCell::new(),
            config:                        OnceCell::new(),
            balances:                      OnceCell::new(),
            delegations:                   OnceCell::new(),
            validator:                     OnceCell::new(),
            config_file:                   OnceCell::new(),
            wallet_path:                   OnceCell::new(),
            nonce_file:                    OnceCell::new(),
            key_file:                      OnceCell::new(),
            address:                       OnceCell::new(),
            export:                        OnceCell::new(),
            balance:                       OnceCell::new(),
            delegations_timestamp:         OnceCell::new(),
            total_staked:                  OnceCell::new(),
            total_liquid:                  OnceCell::new(),
            config_minimum_balance:        OnceCell::new(),
            config_minimum_balance_ratio:  OnceCell::new(),
            config_minimum_stake:          OnceCell::new(),
            config_adjust_minimum_stake:   OnceCell::new(),
            config_minimum_stake_rounding: OnceCell::new(),
            config_daily_reward:           OnceCell::new(),
            config_validator_address:      OnceCell::new(),
            config_validator_moniker:      OnceCell::new(),
            moniker:                       OnceCell::new(),
            voting_power:                  OnceCell::new(),
            rank:                          OnceCell::new(),
            validator_staked:              OnceCell::new(),
            minimum_balance:               OnceCell::new(),
            available_without_claim:       OnceCell::new(),
            available_after_claim:         OnceCell::new(),
            stake_factor:                  OnceCell::new(),
            validator_staked_remainder:    OnceCell::new(),
            can_stake_without_claim:       OnceCell::new(),
            can_stake_after_claim:         OnceCell::new(),
            needs_claim:                   OnceCell::new(),
            quantity_to_stake:             OnceCell::new(),
            daily_reward:                  OnceCell::new(),
            last_journal:                  OnceCell::new(),
            claimed:                       false,
            staked:                        false,
        }
    }

    // Helper function to perform copying of config and wallet data
    fn copy_config_and_wallet<P: AsRef<Path>>(profile_home: P, home: P) -> Result<()> {
        // Copy home/config to profiles_dir/name/config
        let home_config = home.as_ref().join("config");
        let profile_config = profile_home.as_ref().join("config");

        if home_config.exists() {
            fs::copy(&home_config, &profile_config)
                .with_context(|| format!("Failed to copy config file from {:?} to {:?}", home_config, profile_config))?;
        }

        // Copy home/.orga-wallet to profiles_dir/name/.orga-wallet
        let home_wallet = home.as_ref().join(".orga-wallet");
        let profile_wallet = profile_home.as_ref().join(".orga-wallet");

        if home_wallet.exists() {
            fs_extra::dir::copy(&home_wallet, &profile_wallet, &fs_extra::dir::CopyOptions::new())
                .with_context(|| format!(
                    "Failed to copy wallet folder from {:?} to {:?}", 
                    home_wallet, profile_wallet
                ))?;
        }

        Ok(())
    }

    // Helper function to check and copy data from home to profiles directory
    fn check_and_copy_data<P: AsRef<Path>>(profile_home: P, home: P, overwrite: Option<bool>) -> Result<()> {
        let profile_privkey = profile_home.as_ref().join(".orga-wallet/privkey");
        let home_privkey = home.as_ref().join(".orga-wallet/privkey");

        // Check if privkey exists in both locations
        if profile_privkey.exists() && home_privkey.exists() {
            // Handle overwrite decision
            match overwrite {
                Some(true) => {
                    // Proceed with copying data
                    Self::copy_config_and_wallet(profile_home, home)?;
                },
                Some(false) => return Err(eyre!("Cannot overwrite existing profile data.")),
                None => {
                    // Prompt the user if overwrite decision is not provided
                    let user_input = prompt_user("Profile data exists. Do you want to overwrite it?")?;
                    match user_input.as_str() {
                        "y" | "Y" => {
                            // Proceed with copying if the user confirms
                            Self::copy_config_and_wallet(profile_home, home)?;
                        },
                        _ => return Err(eyre!("Profile data exists, and overwrite was not confirmed.")),
                    }
                }
            }
        }

        Ok(())
    }

    pub fn new<P: AsRef<Path>>(
        name: Option<String>,
        home: Option<P>,
        validators: Option<ValidatorCollection>,
        timestamp: Option<DateTime<Utc>>,
        overwrite: Option<bool>,
    ) -> Result<Self> {
        // Set the timestamp to either provided or current
        let timestamp = timestamp.unwrap_or_else(Utc::now);

        // Resolve the profiles directory
        let profiles_dir = &*PROFILES_DIR;

        // Step 1: Handle the case where both `name` and `home` are provided
        if let (Some(name), Some(home)) = (name.clone(), home.as_ref()) {
            let home_path = home.as_ref().to_path_buf();
            let profile_home_path = profiles_dir.join(&name);

            // Check if the resolved `profiles_dir/name` matches the provided `home`
            if home_path != profile_home_path {
                // Check if home is an immediate subdirectory of profiles_dir
                if home_path.starts_with(profiles_dir) {
                    return Err(eyre!("Invalid name: home is an immediate subdirectory of profiles_dir"));
                }
                // Create `profiles_dir/name` if it doesn't exist
                if !profile_home_path.exists() {
                    fs::create_dir_all(&profile_home_path)
                        .with_context(|| format!("Failed to create profile directory: {:?}", profile_home_path))?;
                }

                // Perform overwrite checks and file copying
                Self::check_and_copy_data(&profile_home_path, &home_path, overwrite)?;
            }
            return Ok(Self::create_profile(
                timestamp,          // timestamp
                name,               // name
                profile_home_path,  // home
                validators,         // validators
            ));
        }

        // Step 2: If only `name` is provided
        if let Some(name) = name.clone() {
            let profile_home_path = profiles_dir.join(&name);

            // Create `profiles_dir/name` if it doesn't exist
            if !profile_home_path.exists() {
                fs::create_dir_all(&profile_home_path)
                    .with_context(|| format!("Failed to create profile directory: {:?}", profile_home_path))?;
            }
            return Ok(Self::create_profile(
                timestamp,          // timestamp
                name,               // name
                profile_home_path,  // home
                validators,         // validators
            ));
        }

        // Step 3: If only `home` is provided
        if let Some(home) = home {
            let home_path = home.as_ref().to_path_buf();
            let home_name = home_path.file_name()
                .ok_or_else(|| eyre!("Invalid home directory: unable to extract name"))?
                .to_string_lossy()
                .to_string();
            let profile_home_path = profiles_dir.join(&home_name);

            return Ok(Self::create_profile(
                timestamp,          // timestamp
                home_name,          // name
                profile_home_path,  // home
                validators,         // validators
            ));
        }

        // Step 4: If both `name` and `home` are `None`, return an error
        Err(eyre!("Both `name` and `home` cannot be `None`"))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    /// Lazily initialize and return the wallet path
    pub fn wallet_path(&self) -> Result<&Path> {
        self.wallet_path.get_or_try_init(|| {
            let wallet_path = self.home().join(".orga-wallet");

            if !wallet_path.exists() {
                fs::create_dir_all(&wallet_path)
                    .wrap_err_with(|| format!("Failed to create directory: {:?}", wallet_path))?;
            }

            Ok(wallet_path)
        }).map(|path| path.as_path())
    }

    /// Returns the config file path.
    /// We just want the path here, we leave verification to the Config struct
    pub fn config_file(&self) -> Result<&Path> {
        self.config_file.get_or_try_init(|| {
            Ok(self.home().join("config"))
        }).map(|path| path.as_path())
    }

    /// Returns the key file path.
    /// We only need the path, verification left to the Key struct
    pub fn key_file(&self) -> Result<&Path> {
        self.key_file.get_or_try_init(|| {
            Ok(self.wallet_path()?.join("privkey"))
        }).map(|path| path.as_path())
    }

    /// Returns the nonce file path.
    /// only need the path all other stuff by the Nonce module
    pub fn nonce_file(&self) -> Result<&Path> {
        self.nonce_file.get_or_try_init(|| {
            Ok(self.wallet_path()?.join("nonce"))
        }).map(|path| path.as_path())
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
    pub fn address(&self) -> eyre::Result<&str> {
        self.address.get_or_try_init(|| {
            Ok(self.key()?.address()?.to_string())
        }).map(|s| s.as_str())
    }

    /// Get the export
    /// Operation by cosmrs cache with OnceCell
    /// cached in OnceCell as self.key.export() -> Result<&str>
    /// use self.key.export() to propagate errors
    /// this for display purposes
    pub fn export(&self) -> eyre::Result<&str> {
        self.export.get_or_try_init(|| {
            Ok(self.key()?.export()?.to_string())
        }).map(|s| s.as_str())
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
            Delegations::fetch(Some(self.home()))
        })
    }

    /// Retrieves validators, initializing it if necessary.
    /// blockchain operation, cache with oncecell
    pub fn validators(&self) -> eyre::Result<&ValidatorCollection> {
        self.validators.get_or_try_init(|| {
            ValidatorCollection::fetch()
        })
    }

    /// Retrieves config, initializing it if necessary.
    /// file read operation, cache with OnceCell
    pub fn config(&self) -> Result<&Config> {
        self.config.get_or_try_init(|| {
            Config::new(
                Some(self.name()),                // profile name
                self.config_file()?,              // config file
                Some(self.validators()?.clone()), // validator collecttion already retrieved
            )
        })
    }

    /// import a new private key into profile
    pub fn import(&self, hex_str: &str, force: bool) -> Result<()> {
        let key_file = self.key_file()?; // Get the key file path

        // Check if the key file already exists
        if key_file.exists() && !force {
            return Err(eyre::eyre!("Key file already exists. Use 'force' to overwrite it."));
        }

        // Import the private key from the hex string
        let key = PrivKey::import(
            hex_str,  // hex string
            None      // generate its own timestamp
        )?;           // propagate errors

        // Save the key to the key file
        key.save(key_file, force)?;

        Ok(())
    }

    pub fn export_nonce(&self) -> Result<u64> {
        let nonce_file = self.nonce_file()?;
        nonce::export(Some(&nonce_file), None)
    }

    /// balance -> u64
    /// self.balances()?.nom ->Result<u64>
    fn balance_result(&self) -> Result<&u64> {
            Ok(&self.balances()?.nom)
    }
    pub fn balance(&self) -> &u64 {
        self.balance.get_or_init(|| {
            *self.balance_result().unwrap_or(&ZERO)
        })
    }

    /// delegations_timestamp -> DateTime<Utc>
    /// self.delegations()?.timestamp -> Result<DateTime<Utc>>
    pub fn delegations_timestamp(&self) -> Result<&DateTime<Utc>> {
        self.delegations_timestamp.get_or_try_init(|| {
            Ok(self.delegations()?.timestamp)
        })
    }

    /// total_staked -> u64
    /// self.delegations()?.total().staked -> Result<u64>
    fn total_staked_result(&self) -> Result<u64> {
        Ok(self.delegations()?.total().staked)
    }
    pub fn total_staked(&self) -> &u64 {
        self.total_staked.get_or_init(|| {
            self.total_staked_result().unwrap_or(ZERO)
        })
    }

    /// total_liquid -> u64
    /// self.delegations()?.total().liquid -> Result<u64>
    fn total_liquid_result(&self) -> Result<u64> {
        Ok(self.delegations()?.total().liquid)
    }
    pub fn total_liquid(&self) -> &u64 {
        self.total_liquid.get_or_init(|| {
            self.total_liquid_result().unwrap_or(ZERO)
        })
    }

    /// config_minimum_balance -> u64
    /// self.config()?.minimum_balance -> Result<u64>
    pub fn config_minimum_balance_result(&self) -> Result<u64> {
        Ok(*self.config()?.minimum_balance())
    }
    pub fn config_minimum_balance(&self) -> &u64 {
        self.config_minimum_balance.get_or_init(|| {
            self.config_minimum_balance_result().unwrap_or(100_000)
        })
    }

    /// config_minimum_balance_ratio -> f64
    /// self.config()?.minimum_balance_ratio -> Result<f64>
    pub fn config_minimum_balance_ratio_result(&self) -> Result<f64> {
        Ok(*self.config()?.minimum_balance_ratio())
    }
    pub fn config_minimum_balance_ratio(&self) -> &f64 {
        self.config_minimum_balance_ratio.get_or_init(|| {
            self.config_minimum_balance_ratio_result().unwrap_or(0.001)
        })
    }

    /// config_minimum_stake -> u64
    /// self.config()?.minimum_stake -> Result<u64>
    pub fn config_minimum_stake_result(&self) -> Result<u64> {
        Ok(*self.config()?.minimum_stake())
    }
    pub fn config_minimum_stake(&self) -> &u64 {
        self.config_minimum_stake.get_or_init(|| {
            self.config_minimum_stake_result().unwrap_or(1_000_000)
        })
    }

    /// config_adjust_minimum_stake -> bool
    /// self.config()?.adjust_minimum_stake -> Result<bool>
    pub fn config_adjust_minimum_stake_result(&self) -> Result<bool> {
        Ok(*self.config()?.adjust_minimum_stake())
    }
    pub fn config_adjust_minimum_stake(&self) -> &bool {
        self.config_adjust_minimum_stake.get_or_init(|| {
            self.config_adjust_minimum_stake_result().unwrap_or(false)
        })
    }

    /// config_minimum_stake_rounding -> u64
    /// self.config()?.minimum_stake_rounding -> Result<u64>
    pub fn config_minimum_stake_rounding_result(&self) -> Result<u64> {
        Ok(*self.config()?.minimum_stake_rounding())
    }
    pub fn config_minimum_stake_rounding(&self) -> &u64 {
        self.config_minimum_stake_rounding.get_or_init(|| {
            self.config_minimum_stake_rounding_result().unwrap_or(10_000)
        })
    }

    /// config_daily_reward -> f64
    /// self.config()?.daily_reward -> Result<f64>
    pub fn config_daily_reward_result(&self) -> Result<f64> {
        Ok(*self.config()?.daily_reward())
    }
    pub fn config_daily_reward(&self) -> &f64 {
        self.config_daily_reward.get_or_init(|| {
            self.config_daily_reward_result().unwrap_or(0.0)
        })
    }

    /// config_validator_address -> &str
    /// self.config()?.active_validator()?.address -> Result<&str>
    fn config_validator_address_result(&self) -> Result<&str> {
        Ok(&self.config()?.active_validator()?.address)
    }
    pub fn config_validator_address(&self) -> &str {
        self.config_validator_address.get_or_init(|| {
            self.config_validator_address_result().unwrap_or("").to_string()
        })
    }

    /// config_moniker -> &str
    /// self.config()?.active_validator()?.moniker -> Result<&str>
    pub fn config_validator_moniker_result(&self) -> Result<&str> {
        Ok(&self.config()?.active_validator()?.moniker)
    }
    pub fn config_validator_moniker(&self) -> &str {
        self.config_validator_moniker.get_or_init(|| {
            self.config_validator_moniker_result().unwrap_or("").to_string()
        })
    }

    /// self.validators()?.validator(self.config_validator_address())? -> Result<Validator>
    pub fn validator(&self) -> eyre::Result<&Validator> {
        self.validator.get_or_try_init(|| {
            Ok(self.validators()?.validator(self.config_validator_address())?.clone())
        })
    }

    /// Retrieves the moniker of the validator as a String.
    pub fn moniker_result(&self) -> Result<&str> {
            Ok(&self.validator()?.moniker())
    }
    pub fn moniker(&self) -> &str {
        self.moniker.get_or_init(|| {
            self.moniker_result().unwrap_or("").to_string()
        })
    }

    /// Retrieves the voting_power of the validator as a String.
    pub fn voting_power_result(&self) -> Result<u64> {
        Ok(self.validator()?.voting_power())
    }
    pub fn voting_power(&self) -> &u64 {
        self.voting_power.get_or_init(|| {
            self.voting_power_result().unwrap_or(ZERO)
        })
    }

    /// rank -> u64
    pub fn rank_result(&self) -> Result<u64> {
        Ok(self.validator()?.rank())
    }
    pub fn rank(&self) -> &u64 {
        self.rank.get_or_init(|| {
            self.rank_result().unwrap_or(ZERO)
        })
    }

    /// validator_staked -> u64
    /// self.delegations()?.find(self.config_validator_address()?)?.staked -> Result<u64>
    pub fn validator_staked_result(&self) -> Result<&u64> {
        Ok(&self.delegations()?.find(self.config_validator_address())?.staked)
    }
    pub fn validator_staked(&self) -> &u64 {
        self.validator_staked.get_or_init(|| {
            *self.validator_staked_result().unwrap_or(&ZERO)
        })
    }

    pub fn claim_fee(&self) -> u64 {
        (*CLAIM_FEE * 1_000_000.0) as u64
    }

    pub fn stake_fee(&self) -> u64 {
        (*STAKE_FEE * 1_000_000.0) as u64
    }

    /// Retrieves the validator address based on an optional search string.
    /// 
    /// This function attempts to resolve a validator address by searching through:
    /// 1. A provided search string, which could be a validator moniker or an address.
    /// 2. The config file, assuming the last validator listed as the active one if no search string is provided.
    /// 3. Profiles or other address sources, falling back to checking if the search string is directly a validator or address.
    /// 
    /// # Parameters
    /// 
    /// - `search_str`: An optional search string, which can be a validator moniker or address.
    ///     - If `Some(search_str)` is provided, it will attempt to resolve the address based on the input.
    ///     - If `None`, the function defaults to using the last validator listed in the config (assumed to be the active validator).
    /// 
    /// # Returns
    /// 
    /// - Returns the resolved validator address as a `Result<&str>`.
    ///     - On success, the address of the validator is returned.
    ///     - On failure, an error is returned if no valid validator can be found.
    /// 
    /// # Errors
    /// 
    /// - Returns an error if the search string does not resolve to a valid validator address.
    /// - Propagates errors if the config, profile, or validators cannot be accessed or contain invalid data.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// // Retrieve the validator address based on a specific search string
    /// let address = instance.validator_address(Some("validator_moniker"))?;
    /// 
    /// // Retrieve the active validator's address from the config when no search string is provided
    /// let address = instance.validator_address(None)?;
    /// ```
    /// 
    /// This method handles both cases where a search string is provided and where the active validator is used by default.
    fn validator_address(&self, search_str: Option<&str>) -> eyre::Result<String> {
        // Handle the optional search string
        let search = match search_str {
            Some(v) => v,
            None => self.config_validator_address(),
        };

        // Assume search is a config file moniker and get the associated address from the config
        let address = match self.config()?.search_validator(&search) {
            Ok(a) => {
                // If we get an address, confirm it's actually a validator
                self.validators()?.validator(a)?.address().to_string()
            },
            Err(_) => {
                // Search didn't match a config moniker, assume it's a profile, address, or home folder
                match ProfileCollection::new()?.address(Some(search)) {
                    Ok(a) => {
                        // If we get an address, confirm it's actually a validator
                        self.validators()?.validator(&a)?.address().to_string()
                    },
                    Err(_) => {
                        // Final fallback: assume it's either a validator moniker or address
                        self.validators()?.validator(&search)?.address().to_string()
                    }
                }
            },
        };

        Ok(address)
    }


    pub fn minimum_balance_result(&self) -> Result<u64> {
        // Calculate the minimum required balance based on the ratio
        let calculated_min = (
            self.total_staked_result()? as f64 *
            self.config_minimum_balance_ratio_result()?
        ).floor() as u64;
        let min_bal = self.config_minimum_balance_result()?;

        // Return the maximum of the calculated minimum and configured minimum
        Ok(std::cmp::max(min_bal, calculated_min))
    }

    pub fn minimum_balance(&self) -> &u64 {
        self.minimum_balance.get_or_init(|| {
            self.minimum_balance_result().unwrap_or(100_000)
        })

    }

    fn available_without_claim_result(&self) -> Result<u64> {
        // Calculate available amount before claim
        Ok(self.balance_result()?
            .saturating_sub(self.minimum_balance_result()?)
            .saturating_sub(self.stake_fee())
            .max(0)
        )
    }
    pub fn available_without_claim(&self) -> &u64 {
        self.available_without_claim.get_or_init(|| {
            self.available_without_claim_result().unwrap_or(ZERO)
        })
    }

    pub fn available_after_claim_result(&self) -> Result<u64> {
        // Calculate available amount after the claim
        Ok(self.balance_result()?
            .saturating_add(self.total_liquid_result()?)
            .saturating_sub(self.minimum_balance_result()?)
            .saturating_sub(self.claim_fee())
            .saturating_sub(self.stake_fee())
            .max(0)
        )
    }
    pub fn available_after_claim(&self) -> &u64 {
        self.available_after_claim.get_or_init(|| {
            self.available_after_claim_result().unwrap_or(0)
        })
    }

    fn stake_factor_result(&self) -> Result<u64> {
        // Calculate the stake factor based on the rounding logic
        let stake = self.config_minimum_stake_result()?;
        let round = self.config_minimum_stake_rounding_result()?;

        if round == 0 || stake < round {
            Ok(stake)
        } else {
            Ok(stake.saturating_div(round).saturating_mul(round))
        }
    }

    pub fn stake_factor(&self) -> &u64 {
        self.stake_factor.get_or_init(|| {
            self.stake_factor_result().unwrap_or(10_000)
        })
    }

    pub fn validator_staked_remainder(&self) -> &u64 {
        self.validator_staked_remainder.get_or_init(|| {
            self.validator_staked() % *self.stake_factor()
        })
    }

    /// Fetch the last journal entry for the current executable related to a specific address.
    ///
    /// This function executes the `journalctl` command to retrieve the last log entry
    /// associated with the given address and the running instance of the executable.
    /// It returns the parsed JSON representation of the log entry.
    pub fn last_journal(&self) -> Result<&serde_json::Value> {
        self.last_journal.get_or_try_init(|| {
            let address = self.address()?;

            // Prepare the grep expression, escaping necessary characters
            let grep_expr = format!(r#"{{.*"address"[[:space:]]*:[[:space:]]*"{}".*}}"#, address);

            // Get the current executable path
            let exe_path = env::current_exe()
               .wrap_err("Failed to get the current executable path")?;

            // Convert the path to a string
            let exe_path_str = exe_path.to_string_lossy();
            //let exe_path_str = "/usr/local/bin/nomic-tools".to_string();

            // println!("{}", &exe_path_str);
            // println!("{}", &grep_expr);

            // Use the executable path with journalctl
            let output = Command::new("journalctl")
                .args(&[
                    &format!("_EXE={}", exe_path_str),
                    &format!("--grep={}", &grep_expr),
                    "--output=cat",
                    "--no-pager",
                    "--reverse",
                    "--lines=1",
                ])
                .output()
                .wrap_err("Failed to execute journalctl command")?;

            // Check if the command executed successfully and has output
            if !output.status.success() {
                return Err(eyre::eyre!("journalctl command failed with status: {}", output.status));
            }

            // Check if there's output
            if output.stdout.is_empty() {
                return Err(eyre::eyre!("No output from journalctl command"));
            }

            // Convert the output to a string and parse it as JSON
            let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let json: Value = serde_json::from_str(&output_str)
                .wrap_err("Failed to parse output as JSON")?;

            Ok(json)
        })
    }

    pub fn daily_reward_result(&self) -> Result<f64> {
        self.daily_reward.get_or_try_init(|| {
            // Fetch the last journal entry
            let last_journal = self.last_journal()?;

            // Extract and validate data from the journal
            let last_total_staked = last_journal
                .get("total_staked")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| eyre::eyre!("Missing or invalid 'total_staked' in journal"))?;

            let last_total_liquid = last_journal
                .get("total_liquid")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| eyre::eyre!("Missing or invalid 'total_liquid' in journal"))?;

            let last_timestamp_str = last_journal
                .get("timestamp")
                .and_then(|v| v.as_str())
                .ok_or_else(|| eyre::eyre!("Missing or invalid 'timestamp' in journal"))?;

            // Parse the last timestamp from string to DateTime
            let last_timestamp = DateTime::parse_from_rfc3339(last_timestamp_str)
                .map_err(|_| eyre::eyre!("Failed to parse 'timestamp' as RFC3339"))?
                .timestamp();

            // Extract current data
            let current_total_staked = self.total_staked_result()?;
            let current_total_liquid = self.total_liquid_result()?;
            let current_timestamp = self.delegations_timestamp()?.timestamp();

            // Check conditions and return descriptive errors if they fail
            if last_total_staked != current_total_staked {
                return Err(eyre::eyre!(
                    "Total staked mismatch: last={}, current={}",
                    last_total_staked,
                    current_total_staked
                ));
            }
            if last_total_liquid >= current_total_liquid {
                return Err(eyre::eyre!(
                    "No liquid increase: last={}, current={}",
                    last_total_liquid,
                    current_total_liquid
                ));
            }
            if last_timestamp >= current_timestamp {
                return Err(eyre::eyre!(
                    "Invalid timestamp: last={}, current={}",
                    last_timestamp,
                    current_timestamp
                ));
            }

            // Calculate the deltas
            let reward_delta = current_total_liquid - last_total_liquid;
            let time_delta = current_timestamp - last_timestamp;

            // Calculate the daily reward
            let daily_reward = (reward_delta as f64 / time_delta as f64 * 86400.0).round();

            let mut config = self.config()?.clone();
            config.set_daily_reward(daily_reward);


            Ok(daily_reward)
        }).cloned()
    }

    pub fn daily_reward(&self) -> f64 {
        match self.daily_reward_result() {
            Ok(v) => v,
            Err(_e) => {
                // eprintln!("Failed to calculate daily reward: {:?}", e); // Logging the error
                0.0
            }
        }
    }

    pub fn set_daily_reward(&mut self) -> Result<()> {
        let daily_reward = self.daily_reward_result()?;
        let mut config = self.config()?.clone();
        config.set_daily_reward(daily_reward);
        self.config = OnceCell::from(config.clone());
        Ok(())
    }

    pub fn can_stake_without_claim(&self) -> &bool {
        self.can_stake_without_claim.get_or_init(|| {
            let factor    = *self.stake_factor();
            let available = *self.available_without_claim();
            let remainder = *self.validator_staked_remainder();

            // Determine if staking can occur without needing to claim
            if remainder > 0 {
                available > remainder
            } else {
                available > factor
            }
        })
    }

    pub fn can_stake_after_claim(&self) -> &bool {
        self.can_stake_after_claim.get_or_init(|| {
        let factor    = *self.stake_factor();
        let available = *self.available_after_claim();
        let remainder = *self.validator_staked_remainder();
        let liquid    = *self.total_liquid();
        let claim_fee = self.claim_fee();

        (liquid > claim_fee)
            .then_some(remainder)
            .map(|rem| if rem > 0 { available > rem } else { available > factor })
            .unwrap_or(false)
        })
    }

    pub fn needs_claim(&self) -> &bool {
        self.needs_claim.get_or_init(|| {
            !*self.can_stake_without_claim() &&
             *self.can_stake_after_claim()
        })
    }

    pub fn quantity_to_stake(&self) -> &u64 {
        self.quantity_to_stake.get_or_init(|| {
            let can_stake_without_claim = *self.can_stake_without_claim();
            let can_stake_after_claim = *self.can_stake_after_claim();
            let available_without_claim = *self.available_without_claim();
            let available_after_claim = *self.available_after_claim();
            let stake_factor = *self.stake_factor();
            let validator_staked_remainder = *self.validator_staked_remainder();

            // Determine the available amount to stake based on conditions
            let available_to_stake = if can_stake_without_claim {
                available_without_claim
            } else if can_stake_after_claim {
                available_after_claim
            } else {
                // If neither staking condition is met, return 0 for staking
                return ZERO;
            };

            // Calculate how much is needed to round `validator_staked` to a multiple of `stake_factor`
            let needed_to_round = if validator_staked_remainder == 0 {
                0
            } else {
                stake_factor.saturating_sub(validator_staked_remainder)
            };

            // Check if there's enough available to cover the rounding amount
            if available_to_stake >= needed_to_round {
                // Calculate how much remains after rounding the validator stake
                let remaining_after_round = available_to_stake.saturating_sub(needed_to_round);

                // Determine how many full stake_factor multiples can be staked after rounding
                let multiples_of_stake_factor = remaining_after_round.saturating_div(stake_factor);

                // The final stake amount is `needed_to_round` plus the maximum multiples of `stake_factor`
                needed_to_round.saturating_add(multiples_of_stake_factor.saturating_mul(stake_factor))
            } else {
                // Not enough to cover rounding, return zero
                ZERO
            }
        })
    }
}

// Custom Debug implementation for Profile
impl std::fmt::Debug for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Profile {{ address: {}, name: {} }}",
            self.address().map_err(|_| std::fmt::Error)?,
            self.name()
        )
    }
}

impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        match (self.address(), other.address()) {
            (Ok(addr1), Ok(addr2)) => addr1 == addr2, // Both addresses are OK, compare them
            _ => false, // If either address returns an error, they are not equal
        }
    }
}

impl Profile {

    pub fn nomic_claim(&mut self) -> eyre::Result<()> {

        let _ = self.set_daily_reward();

        // Create and configure the Command for running "nomic claim"
        let mut cmd = Command::new(&*NOMIC);
        cmd.arg("claim");

        // Set the environment variables for NOMIC_LEGACY_VERSION
        cmd.env("NOMIC_LEGACY_VERSION", &*NOMIC_LEGACY_VERSION);

        // Set the HOME environment variable
        let home_path: &OsStr = self.home().as_os_str();
        cmd.env("HOME", home_path);

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
        self.claimed = true;

        Ok(())
    }

    pub fn nomic_delegate(
        &mut self,
        validator: Option<String>,
        quantity: Option<f64>,
    ) -> eyre::Result<()> {

        let _ = self.set_daily_reward();

        let validator_address = match self.validator_address(validator.as_deref()) {
            Ok(address) => address,
            Err(e) => {
                self.print(Some(OutputFormat::Json))?;
                return Err(eyre!("Failed to resolve validator address: {}", e));
            }
        };

        let calc_qty = self.quantity_to_stake();

        let quantity = match quantity {
            Some(q) => ( q * 1_000_000.0 ) as u64,
            None    => *calc_qty,
        };

        if quantity <= 0 {
            self.print(Some(OutputFormat::Json))?;
            return Err(eyre!("Quantity to stake must be greater than 0."));
        }

        if quantity > *self.available_without_claim() &&
           quantity > *self.available_after_claim()
        {
            self.print(Some(OutputFormat::Json))?;
            return Err(eyre!("Not enough balance to stake that quantity."));
        }

        if quantity > *self.available_without_claim() {
            if let Err(e) = self.nomic_claim() {
                self.print(Some(OutputFormat::Json))?;
                return Err(eyre!("Failed to claim: {:?}", e));
            }
            self.claimed = true;
        }

        // let validator = self.config_validator_address();
        // Create and configure the Command for running "nomic delegate"
        let mut cmd = Command::new(&*NOMIC);

        // Set the environment variables for NOMIC_LEGACY_VERSION
        cmd.env("NOMIC_LEGACY_VERSION", &*NOMIC_LEGACY_VERSION);

        // Assuming `self.home()` returns a &Path
        let home_path: &OsStr = self.home().as_os_str();
        cmd.env("HOME", home_path);

        // Add the "delegate" argument, validator, and quantity
        cmd.arg("delegate");
        cmd.arg(validator_address);
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
            self.print(Some(OutputFormat::Json))?;
            return Err(eyre!(error_msg));
        }
        self.staked = true;

        // Clone the config
        let mut config = self.config()?.clone();

        // Rotate the config validators
        let _ = config.rotate_validators()?;

        // Save config to disk
        config.save(self.config_file()?, true)?;

        self.print(Some(OutputFormat::Json))?;
        Ok(())

    }

    pub fn redelegate(
        &self,
        source: &str,
        destination: &str,
        quantity: f64,
    ) -> eyre::Result<()> {

        let source_address = self.validator_address(Some(source))?;
        let destination_address = self.validator_address(Some(destination))?;

        let quantity = (quantity * 1_000_000.0) as u64;

        // let validator = self.config_validator_address();
        // Create and configure the Command for running "nomic delegate"
        let mut cmd = Command::new(&*NOMIC);

        // Set the environment variables for NOMIC_LEGACY_VERSION
        cmd.env("NOMIC_LEGACY_VERSION", &*NOMIC_LEGACY_VERSION);

        // Assuming `self.home()` returns a &Path
        let home_path: &OsStr = self.home().as_os_str();
        cmd.env("HOME", home_path);

        // Add the "delegate" argument, validator, and quantity
        cmd.arg("redelegate");
        cmd.arg(source_address);
        cmd.arg(destination_address);
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

        Ok(())

    }

    pub fn nomic_send(
        &self,
        destination_address: String,
        quantity: Option<f64>,
    ) -> eyre::Result<()> {

        if !is_valid_nomic_address(&destination_address) {
            return Err(eyre!("Invalid address: {}", &destination_address));
        }

        let available = self.balances()?.nom
            .saturating_sub(self.stake_fee().saturating_mul(10));

        let quantity = match quantity {
            Some(q) => (q * 1_000_000.0) as u64,
            None => available,
        };

        if quantity > available {
            return Err(eyre!("Not enough to send"));
        }

        // let validator = self.config_validator_address();
        // Create and configure the Command for running "nomic delegate"
        let mut cmd = Command::new(&*NOMIC);

        // Set the environment variables for NOMIC_LEGACY_VERSION
        cmd.env("NOMIC_LEGACY_VERSION", &*NOMIC_LEGACY_VERSION);

        // Assuming `self.home()` returns a &Path
        let home_path: &OsStr = self.home().as_os_str();
        cmd.env("HOME", home_path);

        // Add the "delegate" argument, validator, and quantity
        cmd.arg("send");
        cmd.arg(destination_address);
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

        Ok(())

    }

//  pub fn auto_delegate(&self, log: bool) -> eyre::Result<()> {
//      let quantity = *self.quantity_to_stake();
//      let claim = *self.needs_claim();

//      // Log the current state if requested
//      if log {
//          self.print(Some(OutputFormat::Json))?;
//      }

//      if quantity > 0 {
//          if claim {
//              self.nomic_claim()?;
//          }
//          self.nomic_delegate(None, None)?;
//      } else {
//          // Log a message since this is not an error
//          eprintln!("Not enough to stake");
//      }
//      Ok(())
//  }

}
    pub fn result_to_json_string<T, E>(result: Result<T, E>) -> String
    where
        T: ToString,
        E: ToString,
    {
        result.map(|value| value.to_string())
              .unwrap_or_else(|err| format!("Error: {}", err.to_string()))
    }

impl Profile {
    pub fn json(&self) -> eyre::Result<Value> {
        let json_output = json!({
            "profile":                       self.name(),
            "address":                       result_to_json_string(self.address()),
            "balance":                       self.balance(),
            "total_staked":                  self.total_staked(),
            "timestamp":                     result_to_json_string(self.delegations_timestamp().map(|t| t.to_rfc3339())),
            "total_liquid":                  self.total_liquid(),
            "config_minimum_balance":        self.config_minimum_balance(),
            "config_minimum_balance_ratio":  self.config_minimum_balance_ratio(),
            "config_minimum_stake":          self.config_minimum_stake(),
            "config_adjust_minimum_stake":   self.config_adjust_minimum_stake(),
            "config_minimum_stake_rounding": self.config_minimum_stake_rounding(),
            "config_daily_reward":           self.config_daily_reward(),
            "config_validator_address":      self.config_validator_address(),
            "config_validator_moniker":      self.config_validator_moniker(),
            "moniker":                       self.moniker(),
            "voting_power":                  self.voting_power(),
            "rank":                          self.rank(),
            "validator_staked":              self.validator_staked(),
            "claim_fee":                     self.claim_fee(),
            "stake_fee":                     self.stake_fee(),
            "minimum_balance":               self.minimum_balance(),
            "stake_factor":                  self.stake_factor(),
            "available_without_claim":       self.available_without_claim(),
            "available_after_claim":         self.available_after_claim(),
            "validator_staked_remainder":    self.validator_staked_remainder(),
            "can_stake_without_claim":       self.can_stake_without_claim(),
            "can_stake_after_claim":         self.can_stake_after_claim(),
            "daily_reward":                  self.daily_reward(),
            "needs_claim":                   self.needs_claim(),
            "quantity_to_stake":             self.quantity_to_stake(),
            "claimed":                       TaskStatus::from_bool(self.claimed).to_symbol(),
            "staked":                        TaskStatus::from_bool(self.staked).to_symbol(),
        });

        Ok(json_output)
    }

}

/// Enum to represent output formats
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    JsonPretty,
    Table,
}

impl FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json"        => Ok(OutputFormat::Json),
            "json-pretty" => Ok(OutputFormat::JsonPretty),
            "table"       => Ok(OutputFormat::Table),
            _             => Err(format!("Invalid output format: {}", s)),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            OutputFormat::Json       => "json",
            OutputFormat::JsonPretty => "json-pretty",
            OutputFormat::Table      => "table",
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
            OutputFormat::Table => {
                json_table(self.json()?)?.printstd();
            },
        }

        Ok(())
    }


    pub fn edit_config(
        &mut self,
        minimum_balance: Option<u64>,
        minimum_balance_ratio: Option<f64>,
        minimum_stake: Option<u64>,
        adjust_minimum_stake: Option<bool>,
        minimum_stake_rounding: Option<u64>,
        rotate_validators: bool,
        remove_validator: Option<&str>,
        add_validator: Option<&str>,
    ) -> eyre::Result<()> {
        // Check if all inputs are None, return an error
        if minimum_balance.is_none()
            && minimum_balance_ratio.is_none()
            && minimum_stake.is_none()
            && adjust_minimum_stake.is_none()
            && minimum_stake_rounding.is_none()
            && !rotate_validators
            && remove_validator.is_none()
            && add_validator.is_none()
        {
            return Err(eyre!("At least one input must be provided to edit the config."));
        }

        // Clone the config to modify it
        let mut config = self.config()?.clone();

        // Apply changes only if the corresponding option is provided
        if let Some(balance) = minimum_balance {
            config.set_minimum_balance(balance);
        }
        if let Some(ratio) = minimum_balance_ratio {
            config.set_minimum_balance_ratio(ratio);
        }
        if let Some(stake) = minimum_stake {
            config.set_minimum_stake(stake);
        }
        if let Some(adjust_stake) = adjust_minimum_stake {
            config.set_adjust_minimum_stake(adjust_stake);
        }
        if let Some(rounding) = minimum_stake_rounding {
            config.set_minimum_stake_rounding(rounding);
        }
        if rotate_validators {
            let _ = config.rotate_validators();
        }
        if let Some(search) = remove_validator {
            let _ = config.remove_validator(search)?;
        }
        if let Some(validator) = add_validator {
            config.add_validator(validator)?;
        }

        // Update the internal state with the new list of validators.
        self.config = OnceCell::from(config.clone());

        // Save the updated config if needed
        config.save(self.config_file()?, true)?;

        Ok(())
    }
}
