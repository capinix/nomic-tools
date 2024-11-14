
use chrono::{DateTime, Utc, Local};
use crate::functions::format_date_offset;
use crate::functions::format_duration;
use crate::functions::is_valid_nomic_address;
use crate::functions::NumberDisplay;
use crate::functions::prompt_user;
use crate::functions::TableColumns;
use crate::functions::TaskStatus;
use crate::global::CONFIG;
use crate::global::PROFILES_DIR;
use crate::journal::{Journal, OutputFormat};
use crate::nonce::Nonce;
use crate::privkey::PrivKey;
use crate::profiles::Balance;
use crate::profiles::Config;
use crate::profiles::config_filename;
use crate::profiles::Delegation;
use crate::profiles::Delegations;
use crate::profiles::ProfileCollection;
use crate::validators::initialize_validators;
use crate::validators::Validator;
use crate::validators::ValidatorCollection;
use eyre::eyre;
use eyre::Result;
use eyre::WrapErr;
use log::warn;
use once_cell::sync::OnceCell;
use serde_json::Value;
use std::cmp::max;
use std::cmp::PartialEq;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tabled::builder::Builder;
use tabled::settings::{Alignment, Border, Color, Modify, Span, Style};
use tabled::settings::object::{Columns, Rows, Cell};

// Static zero for use when there is an error
// static ZERO: u64 = 0;

#[derive(Clone)]
pub struct Profile {
    name:                          String,
    home:                          PathBuf,
    wallet_path:                   OnceCell<PathBuf>,
    nonce_file:                    OnceCell<PathBuf>,
    key_file:                      OnceCell<PathBuf>,
    key:                           OnceCell<PrivKey>,
    config:                        OnceCell<Config>,
    balances:                      OnceCell<Balance>,
    balance:                       OnceCell<u64>,

    /// ValidatorCollection, used to lookup Validator information
    /// Requires a network request. if previously fetched, and optionally provided,
    /// we might not have to do another network request
    validators:                    OnceCell<ValidatorCollection>,

    validator:                     OnceCell<Validator>,
    delegations:                   OnceCell<Delegations>,
    total_staked:                  OnceCell<u64>,
    total_liquid:                  OnceCell<u64>,
    validator_staked:              OnceCell<u64>,
    delegation:                    OnceCell<Delegation>,
    minimum_balance:               OnceCell<u64>,
    minimum_stake:                 OnceCell<u64>,
    daily_reward:                  OnceCell<u64>,
    last_journal:                  OnceCell<Journal>,
    calc:                          OnceCell<Calc>,
//    available_without_claim:       OnceCell<u64>,
//    available_after_claim:         OnceCell<u64>,
//    validator_staked_remainder:    OnceCell<u64>,
//    needed:                        OnceCell<u64>,
//    can_stake_without_claim:       OnceCell<bool>,
//    can_stake_after_claim:         OnceCell<bool>,
    staked:                        bool,
    claimed:                       bool,
    journal:                       OnceCell<Journal>,
}

impl Profile {

    fn create_profile<P: AsRef<Path>, S: AsRef<str>>(
        name:       S,
        home:       P,
        validators: Option<ValidatorCollection>,
    ) -> Self {
        let name: &str  = name.as_ref();
        let home: &Path = home.as_ref();
        Self {
            name:                         name.to_string(),
            home:                       home.to_path_buf(),
            wallet_path:                   OnceCell::new(),
            nonce_file:                    OnceCell::new(),
            key_file:                      OnceCell::new(),
            key:                           OnceCell::new(),
            config:                        OnceCell::new(),
            balances:                      OnceCell::new(),
            balance:                       OnceCell::new(),
            validators:                    initialize_validators(validators),
            validator:                     OnceCell::new(),
            delegations:                   OnceCell::new(),
            total_staked:                  OnceCell::new(),
            total_liquid:                  OnceCell::new(),
            validator_staked:              OnceCell::new(),
            delegation:                    OnceCell::new(),
            minimum_balance:               OnceCell::new(),
            minimum_stake:                 OnceCell::new(),
            last_journal:                  OnceCell::new(),
            daily_reward:                  OnceCell::new(),
            calc:                          OnceCell::new(),
//            available_without_claim:       OnceCell::new(),
//            available_after_claim:         OnceCell::new(),
//            validator_staked_remainder:    OnceCell::new(),
//            can_stake_without_claim:       OnceCell::new(),
//            can_stake_after_claim:         OnceCell::new(),
//            needed:                        OnceCell::new(),
            claimed:                       false,
            staked:                        false,
            journal:                       OnceCell::new(),
        }
    }

    // Helper function to perform copying of config and wallet data
    fn copy_config_and_wallet<P: AsRef<Path>>(
        profile_home: P,
        home: P
    ) -> Result<()> {
        // Copy home/config to profiles_dir/name/config
        let profile_home: &Path = profile_home.as_ref();
        let home: &Path = home.as_ref();

        let home_config = home.join(config_filename());
        let profile_config = profile_home.join(config_filename());

        if home_config.exists() {
            fs::copy(&home_config, &profile_config).with_context(|| format!(
                "Failed to copy config file from {:?} to {:?}", 
                home_config, profile_config
            ))?;
        }

        // Copy home/.orga-wallet to profiles_dir/name/.orga-wallet
        let home_wallet = home.join(".orga-wallet");
        let profile_wallet = profile_home.join(".orga-wallet");

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
    fn check_and_copy_data<P: AsRef<Path>>(
        profile_home: P,
        home: P,
        overwrite: Option<bool>
    ) -> Result<()> {

        let profile_home: &Path = profile_home.as_ref();
        let home: &Path = home.as_ref();

        let profile_privkey = profile_home.join(".orga-wallet").join("privkey");
        let home_privkey    = home.join(".orga-wallet").join("privkey");

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

    fn create_with_name_and_home<P: AsRef<Path>, S: AsRef<str>>(
        name: S,
        home: P,
        profiles_dir: P,
        overwrite: Option<bool>,
        validators: Option<ValidatorCollection>,
    ) -> Result<Self> {
        let name_str: &str = name.as_ref();
        let home_path: &Path = home.as_ref();
        let profiles_path: &Path = profiles_dir.as_ref();
        let profile_home_path = profiles_path.join(name_str);

        if home_path != profile_home_path {
            if home_path.starts_with(profiles_dir) {
                return Err(eyre!(
                    "Invalid name: home is an immediate subdirectory of profiles_dir"
                ));
            }

            // Create `profiles_dir/name` if it doesn't exist
            if !profile_home_path.exists() {
                fs::create_dir_all(&profile_home_path).with_context(|| format!(
                    "Failed to create profile directory: {:?}",
                    profile_home_path,
                ))?;
            }

            Self::check_and_copy_data(
                profile_home_path.to_path_buf(), home_path.to_path_buf(), overwrite)?;
        }

        Ok(Self::create_profile(name, profile_home_path.to_path_buf(), validators))
    }

    fn create_with_name<P: AsRef<Path>, S: AsRef<str>>(
        name: S,
        profiles_dir: P,
        validators: Option<ValidatorCollection>,
    ) -> Result<Self> {
        let name_str: &str = name.as_ref();
        let profiles_path: &Path = profiles_dir.as_ref();
        let profile_home_path = profiles_path.join(name_str);

        // Create `profiles_dir/name` if it doesn't exist
        if !profile_home_path.exists() {
            fs::create_dir_all(&profile_home_path).with_context(|| format!(
                "Failed to create profile directory: {:?}",
                profile_home_path,
            ))?;
        }

        Ok(Self::create_profile(name, profile_home_path, validators))
    }

    fn create_with_home<P: AsRef<Path>>(
        home: P,
        profiles_dir: P,
        validators: Option<ValidatorCollection>,
    ) -> Result<Self> {
        let home_path: &Path = home.as_ref();
        let home_name = home_path.file_name()
            .ok_or_else(|| eyre!("
                Invalid home directory: unable to extract name"
            ))?
            .to_string_lossy()
            .to_string();
        let profiles_path: &Path = profiles_dir.as_ref();
        let profile_home_path = profiles_path.join(&home_name);

        Ok(Self::create_profile(home_name, profile_home_path, validators))
    }

//    fn copy_privkey_file<P: AsRef<Path>>(
//        privkey_file: P,
//        profile: &Profile,
//    ) -> Result<()> {
//        // Ensure the private key file exists
//        if privkey_file.as_ref().exists() {
//            let dest_path = profile.key_file()?;
//
//            // Copy the private key file to the destination
//            fs::copy(&privkey_file, dest_path)
//                .with_context(|| format!(
//                    "Failed to copy private key file from {:?} to {:?}",
//                    privkey_file.as_ref(),
//                    dest_path
//                ))?;
//        } else {
//            return Err(eyre!(
//                "Private key file does not exist: {:?}",
//                privkey_file.as_ref(),
//            ));
//        }
//        Ok(())
//    }

    pub fn new<S: AsRef<str> + AsRef<Path>>(

        name: Option<S>,
        home: Option<S>,

        // This could be either
        //   1 a filename containing a binary representation of a private key
        //   2 a filename containing a hex representation of a private key
        //   3 a hex string representation of a private key
        privkey: Option<S>,

        // if previous files exist, should we overwrite them
        overwrite: Option<bool>,

        // ValidatorCollection, used to lookup Validator information
        // Requires a network request. if previously fetched, and optionally provided,
        // we might not have to do another network request
        validators: Option<&ValidatorCollection>,

    ) -> Result<Self> {
        let profiles_dir: &Path = &*PROFILES_DIR;

        // If both `name` and `home` are provided
        if let (Some(name), Some(home)) = (name.as_ref(), home.as_ref()) {
            // Convert `home_str` to a `PathBuf`
            let name_str:  &str  = name.as_ref();
            let home_path: &Path = home.as_ref();

            let profile = Self::create_with_name_and_home(
                name_str,
                home_path,
                profiles_dir,
                overwrite,
                validators.cloned(),
            )?;

            if let Some(privkey_val) = privkey.as_ref() {
                let key = PrivKey::import(privkey_val)?;
                key.save(profile.wallet_path()?, true)?;
            }

            return Ok(profile);
        }

        // If only `name` is provided
        if let Some(name) = name.as_ref() {
            let name_str: &str = name.as_ref();
            let profile = Self::create_with_name(name_str.to_string(), profiles_dir, validators.cloned())?;

            if let Some(privkey_val) = privkey.as_ref() {
                let key = PrivKey::import(privkey_val)?;
                key.save(profile.key_file()?, true)?;
            }

            return Ok(profile);
        }

        // If only `home` is provided
        if let Some(home) = home.as_ref() {
            let home_path: &Path = home.as_ref();
            let profile = Self::create_with_home(home_path, profiles_dir, validators.cloned())?;

            if let Some(privkey_val) = privkey.as_ref() {
                let key = PrivKey::import(privkey_val)?;
                key.save(profile.key_file()?, true)?;
            }

            return Ok(profile);
        }

        // If both `name` and `home` are `None`, return an error
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
    pub fn config_file(&self) -> PathBuf {
        self.home().join(config_filename())
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

    /// self.key()?.address()? -> Result<&str>
    pub fn address(&self) -> &str {
        self.key()
            .and_then(|key| key.address())
            .unwrap_or("N/A")
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
            Delegations::fetch(self.key()?.address()?, Some(self.home()))
        })
    }

    /// Retrieves validators, initializing it if necessary.
    /// blockchain operation, cache with oncecell
    pub fn validators(&self) -> eyre::Result<&ValidatorCollection> {
        self.validators.get_or_try_init(|| {
            ValidatorCollection::fetch()
        })
    }

    /// Returns a reference to the config, initializing it if necessary.
    pub fn config(&self) -> &Config {
        self.config.get_or_init(|| {
            match Config::load(self.name(), &self.config_file()) {
                Ok(loaded_config) => loaded_config,
                Err(e) => {
                    warn!("Using default config, could not load config: {}", e);
                    // Return a new default Config
                    Config::new(self.name())
                }
            }
        })
    }

    pub fn edit_config(
        &self,
        minimum_balance:        Option<u64>,
        minimum_balance_ratio:  Option<u64>,
        minimum_stake:          Option<u64>,
        adjust_minimum_stake:   Option<bool>,
        minimum_stake_rounding: Option<u64>,
        daily_reward:           Option<u64>,
        add_validator:          Option<String>,
        remove_validator:       Option<String>,
        rotate_validators:      bool,
    ) -> Result<()> {
        let mut config = self.config().clone();

        if let Some(balance) = minimum_balance {
            config.minimum_balance = balance;
        }
        if let Some(balance_ratio) = minimum_balance_ratio {
            config.minimum_balance_ratio = balance_ratio;
        }
        if let Some(stake) = minimum_stake {
            config.minimum_stake = stake;
        }
        if let Some(adjust) = adjust_minimum_stake {
            config.adjust_minimum_stake = adjust;
        }
        if let Some(rounding) = minimum_stake_rounding {
            config.minimum_stake_rounding = rounding;
        }
        if let Some(reward) = daily_reward {
            config.daily_reward = reward;
        }
        if let Some(address_and_name) = add_validator {
            let parts: Vec<&str> = address_and_name.split(',').collect();
            if parts.len() == 2 {
                config.add_validator(parts[0], parts[1]);
            } else {
                warn!("Expected 'address,name' format, but got '{}'", address_and_name);
            }
        }
        if let Some(search) = remove_validator {
            config.remove_validator(&search)?;
        }
        if rotate_validators {
            config.rotate_validators();
        }

        config.save(&self.config_file(), true)?;
        println!("{}", config);
        Ok(())
    }

    pub fn set_config_minimum_balance(&self, minimum_balance: Option<u64>) -> Result<()> {
        let balance = minimum_balance.unwrap_or_else(|| *self.minimum_balance());
        self.edit_config( Some(balance), None, None, None, None, None, None, None, false)
    }

    pub fn set_config_minimum_stake(&self, minimum_stake: Option<u64>) -> Result<()> {
        let stake = minimum_stake.unwrap_or_else(|| *self.minimum_stake());
        self.edit_config( None, None, Some(stake), None, None, None, None, None, false)
    }

    pub fn set_config_daily_reward(&self, daily_reward: Option<u64>) -> Result<()> {
        let reward = daily_reward.unwrap_or_else(|| self.daily_reward());
        self.edit_config( None, None, None, None, None, Some(reward), None, None, false)
    }

    /// import a new private key into profile
    pub fn import<S: AsRef<str>>(&self, data: S, force: bool) -> Result<()> {
        let key_file = self.key_file()?; // Get the key file path

        // Check if the key file already exists
        if key_file.exists() && !force {
            return Err(eyre::eyre!("Key file already exists. Use 'force' to overwrite it."));
        }

        println!("{:?}", &key_file);

        // Import the private key from the hex string
        let key = PrivKey::import(data)?;

        // Save the key to the key file
        key.save(key_file, force)?;

        Ok(())
    }

    pub fn export_nonce(&self) -> Result<u64> {
        let nonce = Nonce::load(self.nonce_file()?.to_string_lossy())?;
        Ok(nonce.value())
    }

    pub fn balance(&self) -> &u64 {
        self.balance.get_or_init(|| {
            self.balances()
                .map(|balances| balances.nom)
                .unwrap_or(0)
        })
    }

    /// self.delegations()?.timestamp -> Result<DateTime<Utc>>
    pub fn delegations_timestamp_rfc3339(&self) -> String {
        self.delegations()
            .map(|delegations| delegations.timestamp.to_rfc3339())
            .unwrap_or("N/A".to_string())
    }

    pub fn total_staked(&self) -> &u64 {
        self.total_staked.get_or_init(|| {
            let default = 0;
            match self.delegations() {
                Ok(delegations) => delegations.total().staked,
                Err(e) => {
                    warn!("Could not load delegations: {}", e);
                    default
                }
            }
        })
    }

    /// self.delegations()?.total().liquid -> Result<u64>
    pub fn total_liquid(&self) -> &u64 {
        self.total_liquid.get_or_init(|| {
            let default = 0;
            match self.delegations() {
                Ok(delegations) => delegations.total().liquid,
                Err(e) => {
                    warn!("Could not load delegations: {}", e);
                    default
                }
            }
        })
    }


    /// self.validators()?.validator(&self.config()?.active_validator()?.address)? -> Result<Validator>
    /// this is a search let's OnceCell it
    pub fn validator(&self) -> eyre::Result<&Validator> {
        self.validator.get_or_try_init(|| {
            Ok(
                self.validators()?
                    .validator(&self.config().validator_address())?
                    .clone()
            )
        })
    }

    /// lookup active validators real moniker with default on error
    pub fn moniker(&self) -> &str {
        self.validator()
            .map(|validator| validator.moniker())
            .unwrap_or("N/A")
    }

    pub fn name_or_moniker(&self, search: &str) -> &str {
        // First search within `config().validators` by `name` or `address`
        self.config().validators
            .iter()
            .find(|validator| validator.name == search || validator.address == search)
            .map(|validator| validator.name.as_str())
            // If no match, try searching `self.validators()` by `moniker` or `address`
            .or_else(|| {
                // Handle `self.validators()` Result and search within `moniker` or `address`
                self.validators().ok()
                    .and_then(|validators| {
                        validators
                            .iter()
                            .find(|validator| validator.moniker() == search || validator.address() == search)
                            .map(|validator| validator.moniker())
                    })
            })
            // Default to "N/A" if no match is found in either list
            .unwrap_or("N/A")
    }


    /// lookup active validators voting power default to 0, on error
    pub fn voting_power(&self) -> u64 {
        self.validator()
            .map(|validator| validator.voting_power())
            .unwrap_or(0)
    }

    /// lookup active validators rank default to 0, on error
    pub fn rank(&self) -> u64 {
        self.validator()
            .map(|validator| validator.rank())
            .unwrap_or(0)
    }

    /// self.delegations()?.find(self.config()?.active_validator()?.address()) -> Result<Delegation>
    pub fn delegation(&self) -> Result<&Delegation> {
        self.delegation.get_or_try_init(|| {
            Ok(self.delegations()?
                .find(&self.config().validator_address())?
                .clone()
            )
        })
    }

    /// self.delegations()?.find(self.config()?.active_validator()?.address()).staked -> Result<u64>
    pub fn validator_staked(&self) -> &u64 {
        self.validator_staked.get_or_init(|| {
            self.delegation()
                .map(|delegation| delegation.staked)
                .unwrap_or(0)
        })
    }

    pub fn claim_fee(&self) -> u64 {
        CONFIG.claim_fee
    }

    pub fn stake_fee(&self) -> u64 {
        CONFIG.stake_fee
    }

    /// Retrieves the validator address based on an optional search string.
    /// 
    /// This function attempts to resolve a validator address by searching through:
    /// 1. A provided search string, which could be a validator moniker or an address.
    /// 2. The config file, assuming the last validator listed as the active one if no search string is provided.
    /// 3. Profiles or other address sources, falling back to checking 
    ///    if the search string is directly a validator or address.
    ///
    /// # Parameters
    ///
    /// - `search_str`: An optional search string, which can be a validator moniker or address.
    ///     - If `Some(search_str)` is provided, it will attempt to resolve the address based on the input.
    ///     - If `None`, the function defaults to using the last validator 
    ///           listed in the config (assumed to be the active validator).
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
    /// This method handles both cases where a search string is provided and where
    ///  the active validator is used by default.
    fn validator_address(&self, search_str: Option<&str>) -> eyre::Result<String> {
        let search = match search_str {
            Some(matched) => matched,
            None => return Ok(self.config().validator_address().to_string()), // return early
        };

        let address = match self.config().search_validator(&search) {
            Ok(validator) => validator.address.clone(),
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

    /// Returns the minimum balance required for the account based on the configured ratio and delegation status.
    ///
    /// # Description
    /// This method calculates the minimum balance as a function of the `minimum_balance_ratio` from
    /// the configuration and the total staked amount in the account's delegations. The result is adjusted
    /// to meet certain conditions, including rounding down to a base unit (`10,000 unom`, or `0.01 nom`) 
    /// and ensuring it is at least equal to the `minimum_balance` specified in the configuration.
    ///
    /// # Calculations
    /// - `adjusted_ratio`: This is the `minimum_balance_ratio` from the config, clamped to a maximum of `1,000,000`
    ///   to represent a value between `0` and `1` when scaled by a factor of `1,000,000`.
    /// - Minimum balance is derived from the staked amount in the account:
    ///   - `staked * adjusted_ratio / 1_000_000`: Scales the staked amount by the adjusted ratio.
    ///   - `.saturating_div(10_000).saturating_mul(10_000)`: Rounds down the result to the nearest base unit of `10,000 unom`.
    ///   - `.max(config.minimum_balance)`: Ensures the final result is not below the minimum balance specified in the config.
    ///
    /// # Returns
    /// A reference to the computed minimum balance, caching the result for future calls.
    ///
    /// # Errors
    /// If there is an error retrieving delegations, the function defaults to using `config.minimum_balance`.
    ///
    pub fn minimum_balance(&self) -> &u64 {
        self.minimum_balance.get_or_init(|| {
            // Load configuration to retrieve minimum balance ratio and base minimum balance.
            let config = self.config();

            // Adjusted ratio is the minimum of the `minimum_balance_ratio` and 1_000_000
            // This effectively scales the ratio between 0 and 1 when divided by 1,000,000.
            let adjusted_ratio = config.minimum_balance_ratio.min(1_000_000);

            match self.delegations() {
                Ok(d) => {
                    // Calculate the minimum balance based on staked amount and adjusted ratio
                    d.total()
                        .staked
                        .saturating_mul(adjusted_ratio)   // Apply ratio to staked amount
                        .saturating_div(1_000_000)        // Scale down to the original ratio (0-1)
                        .saturating_div(10_000)           // Round down to nearest 10_000 unom
                        .saturating_mul(10_000)           // Ensure 10_000 unom increments
                        .max(config.minimum_balance)      // Ensure result meets or exceeds config minimum
                }
                // If delegation information isn't available, default to config's minimum balance.
                Err(_) => config.minimum_balance,
            }
        })
    }

    /// Fetch the last journal entry for the current executable related to a specific address.
    ///
    /// This function executes the `journalctl` command to retrieve the last log entry
    /// associated with the given address and the running instance of the executable.
    /// It returns the parsed JSON representation of the log entry.
    pub fn last_journal(&self) -> eyre::Result<&Journal> {
        self.last_journal.get_or_try_init(|| {
            let address = self.key()?.address()?;

            // Prepare the grep expression, escaping necessary characters
            let grep_expr = format!(r#"{{.*"address"[[:space:]]*:[[:space:]]*"{}".*}}"#, address);

            // Get the current executable path
            let exe_path = env::current_exe()
                .wrap_err("Failed to get the current executable path")?;

            // Convert the path to a string
            let exe_path_str = exe_path.to_string_lossy();

//          let exe_path_str = "/usr/local/bin/nomic-tools".to_string();

            // Use the executable path with journalctl:w
            //
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

            // Convert the output to a string slice and parse it as IndexMap
            let output_str = String::from_utf8_lossy(&output.stdout);
            let journal  = Journal::from_json_str(&output_str.trim())?;

            // Return the newly constructed IndexMap
            Ok(journal) // Return the constructed IndexMap directly

        })
    }

    /// Estimates the daily staking reward based on recent journal entries and current staking data.
    ///
    /// This function retrieves the last recorded journal entry from the systemd logs to compare
    /// previous staking and liquidity values against the current state. This comparison is used 
    /// to determine the rate of reward accumulation.
    ///
    /// ### Process
    /// - Retrieves the last journal entry, extracting data on `total_staked`, `total_liquid`,
    ///   `quantity`, `timestamp`, `staked`, and `claimed`.
    /// - Compares current and last recorded `total_staked` values to check for discrepancies.
    ///   If the current `total_staked` differs from the expected amount (based on the last 
    ///   journal entry's recorded quantity and staking status), the function returns an error.
    /// - If the `claimed` field in the last journal entry is marked as true (indicating a recent
    ///   reward claim), the expected `total_liquid` is set to zero. Otherwise, the prior liquid 
    ///   amount is used.
    /// - Verifies that the last recorded `timestamp` is earlier than the current timestamp. If not, 
    ///   an error is returned, indicating timestamp inconsistency.
    ///
    /// ### Reward Calculation
    /// - Calculates the difference in `total_liquid` since the last entry, confirming an increase 
    ///   has occurred.
    /// - Divides this difference by the elapsed time between the last and current timestamps to 
    ///   determine the reward accumulation rate, scaled to a daily estimate.
    ///
    /// ### Result and Configuration
    /// - Saves the calculated daily reward to a configuration file. If any failure occurs, a warning
    ///   is logged.
    ///
    /// Returns a `Result<u64>` containing the estimated daily reward in staking units or an error if 
    /// any inconsistency in staking or liquid balances prevents estimation.
    pub fn daily_reward_result(&self) -> Result<u64> {
        self.daily_reward.get_or_try_init(|| {
            // Fetch the last journal entry
            let last_journal = self.last_journal()?;

            // Extract and validate data from the journal
            let last_total_staked = last_journal.get::<u64>("total_staked")
                .ok_or_else(|| eyre::eyre!("Missing or invalid 'total_staked' in last journal"))?;

            let last_total_liquid = last_journal.get::<u64>("total_liquid")
                .ok_or_else(|| eyre::eyre!("Missing or invalid 'total_liquid' in last journal"))?;

            let last_quantity = last_journal.get::<u64>("quantity")
                .ok_or_else(|| eyre::eyre!("Missing or invalid 'quantity' in last journal"))?;

            let last_timestamp = last_journal.get::<DateTime<Utc>>("timestamp")
                .ok_or_else(|| eyre::eyre!("Missing or invalid 'timestamp' in last journal"))?
                .timestamp();

            let last_staked = last_journal.get::<String>("staked")
                .ok_or_else(|| eyre::eyre!("Missing or invalid 'staked' in last journal"))?;

            let last_claimed = last_journal.get::<String>("claimed")
                .ok_or_else(|| eyre::eyre!("Missing or invalid 'claimed' in last journal"))?;

            let is_last_staked   = last_staked   == "✅";
            let is_last_claimed  = last_claimed  == "✅";

            // Extract current data
            let current_total_staked = self.delegations()?.total().staked;
            let current_total_liquid = self.delegations()?.total().liquid;
            let current_timestamp    = self.delegations()?.timestamp.timestamp();

            // Calculate expected total staked based on staking status
            let expected_total_staked = if is_last_staked {
                last_total_staked + last_quantity
            } else {
                last_total_staked
            };

            // Check for staked inconsistency
            if current_total_staked != expected_total_staked {
                return Err(eyre::eyre!(
                    "Cannot determine daily reward\nTotal staked mismatch: last={}, current={}",
                    expected_total_staked,
                    current_total_staked
                ));
            }

            // Define the expected liquid balance, adjusting if the last claim has occurred
            let expected_total_liquid = if is_last_claimed { 0 } else { last_total_liquid };

            // Check for liquid inconsistency
            if expected_total_liquid >= current_total_liquid {
                return Err(eyre::eyre!(
                    "Cannot determine daily reward\nNo liquid increase: last={}, current={}",
                    expected_total_liquid,
                    current_total_liquid
                ));
            }

            if last_timestamp >= current_timestamp {
                return Err(eyre::eyre!(
                    "Cannot determine daily reward\nInvalid timestamp: last={}, current={}",
                    last_timestamp,
                    current_timestamp
                ));
            }

            // Calculate the deltas
            let reward_delta = current_total_liquid.saturating_sub(expected_total_liquid);
            let time_delta = current_timestamp.saturating_sub(last_timestamp);

            // Calculate the daily reward
            let daily_reward = if time_delta > 0 {
                ((reward_delta as f64 * 86_400.0) / time_delta as f64) as u64
            } else {
                0 // or some default value
            };

            match Config::load(self.name(), &self.config_file()) {
                Ok(mut config) => {
                    config.daily_reward = daily_reward;
                    if let Err(e) = config.save(&self.config_file(), true) {
                        warn!("Failed to save config file: {}", e);
                    }
                }
                Err(e) => warn!("Could not load config file: {}", e),
            };

            Ok(daily_reward)
        }).cloned()
    }

    /// Retrieves the last successfully calculated daily reward, or defaults to the stored configuration value.
    /// Logs a warning if the calculation fails, before returning the configuration's stored reward value.
    pub fn daily_reward(&self) -> u64 {
        match self.daily_reward_result() {
            Ok(daily_reward) => daily_reward,
            Err(e) => {
                warn!("Failed to retrieve daily reward: {}. Using stored configuration value instead.", e);
                self.config().daily_reward
            }
        }
    }

    pub fn minimum_stake(&self) -> &u64 {
        self.minimum_stake.get_or_init(|| {
            let mut config = self.config().clone();
            let config_min = config.minimum_stake;
            let rounding   = config.minimum_stake_rounding;
            let adjust     = config.adjust_minimum_stake;

            let min = if adjust && rounding > 0 {
                let daily = self.daily_reward().saturating_add(rounding.saturating_div(2));
                max(config_min, daily.saturating_sub(daily % rounding))
            } else {
                config_min
            };

            if min != config_min {
                config.minimum_balance = *self.minimum_balance();
                config.minimum_stake = min;
                config.daily_reward = self.daily_reward();
                if let Err(_) = config.save(&self.config_file(), true) {
                    warn!("Could not save config file");
                }
            }

            // Return the calculated minimum stake
            min
        })
    }

//    pub fn available_without_claim(&self) -> &u64 {
//        self.available_without_claim.get_or_init(|| {
//            match self.balances() {
//                Ok(balances) => {
//                    balances.nom
//                        .saturating_sub(*self.minimum_balance())
//                        .saturating_sub(self.stake_fee())
//                        .max(0)
//                },
//                Err(_) => 0,
//            }
//        })
//    }
//
//    pub fn available_after_claim(&self) -> &u64 {
//        self.available_after_claim.get_or_init(|| {
//            match self.balances() {
//                Ok(balances) => {
//                    balances.nom
//                    .saturating_add(self.delegations().map_or(0, |d| d.total().liquid))
//                    .saturating_sub(*self.minimum_balance())
//                    .saturating_sub(self.claim_fee())
//                    .saturating_sub(self.stake_fee())
//                    .max(0)
//                },
//                Err(_) => 0,
//            }
//        })
//    }

    pub fn calc_quantity(&self, validator_address: Option<&str>, quantity: Option<u64>) -> Calc {
        let staked = match validator_address {
            Some(address) => {
                self.delegations()
                    .and_then(|delegations| delegations.find(address).map(|d| d.staked))
                    .unwrap_or(0)  // Return 0 if any errors occur in the chain
            },
            None => *self.validator_staked(),
        };

        let available_without_claim = match self.balances() {
            Ok(balances) => balances.nom
                .saturating_sub(*self.minimum_balance())
                .saturating_sub(self.stake_fee())
                .max(0),
            Err(_) => 0,
        };

        let available_after_claim = match self.balances() {
            Ok(balances) => balances.nom
                .saturating_add(self.delegations().map_or(0, |d| d.total().liquid))
                .saturating_sub(*self.minimum_balance())
                .saturating_sub(self.claim_fee())
                .saturating_sub(self.stake_fee())
                .max(0),
            Err(_) => 0,
        };

        let remainder = staked % *self.minimum_stake();
        let needed = quantity.unwrap_or_else(|| self.minimum_stake().saturating_sub(remainder));
        let can_stake_without_claim = available_without_claim > needed;
        let can_stake_after_claim = available_after_claim > needed;
        let remaining = if can_stake_without_claim || can_stake_after_claim {
            0
        } else {
            needed.saturating_sub(available_after_claim)
        };
        let needs_claim = !can_stake_without_claim && can_stake_after_claim;

        // Calculate available funds based on conditions
        let available = if can_stake_without_claim {
            available_without_claim
        } else if can_stake_after_claim {
            available_after_claim
        } else {
            0
        };

        let quantity = quantity.unwrap_or_else(||
            available
                .saturating_sub(needed)
                .saturating_div(*self.minimum_stake())
                .saturating_mul(*self.minimum_stake())
                .saturating_add(needed)
        );

        Calc {
            available_without_claim,
            available_after_claim,
            remainder,
            needed,
            can_stake_without_claim,
            can_stake_after_claim,
            remaining,
            needs_claim,
            quantity,
        }
    }

    pub fn calc(&self) -> &Calc {
        self.calc.get_or_init(|| {
            self.calc_quantity(None, None)
        })
    }

//    pub fn validator_staked_remainder(&self) -> &u64 {
//        self.validator_staked_remainder.get_or_init(|| {
//            self.validator_staked() % *self.minimum_stake()
//        })
//    }


//    // Quantity needed to trigger a staking action
//    pub fn needed(&self) -> &u64 {
//        self.needed.get_or_init(|| {
//            self.minimum_stake().saturating_sub(*self.validator_staked_remainder())
//        })
//    }

//    pub fn can_stake_without_claim(&self) -> &bool {
//        self.can_stake_without_claim.get_or_init(|| {
//            *self.available_without_claim() > *self.needed()
//        })
//    }

//    pub fn can_stake_after_claim(&self) -> &bool {
//        self.can_stake_after_claim.get_or_init(|| {
//            *self.available_after_claim() > *self.needed()
//        })
//    }

}

#[derive(Clone, Debug)]
pub struct Calc {
    available_without_claim:  u64,
    available_after_claim:    u64,
    remainder:                u64,
    needed:                   u64,
    can_stake_without_claim:  bool,
    can_stake_after_claim:    bool,
    remaining:                u64,
    needs_claim:              bool,
    quantity:                 u64,

}

// Custom Display implementation for Profile
impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n\n{}\n\n{}",
            self.report_profile(),
            self.report_config(),
            self.report_config_validators(),
        )
    }
}


// Custom Debug implementation for Profile
impl std::fmt::Debug for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Profile {{ address: {}, name: {} }}",
            self.address(),
            self.name()
        )
    }
}

impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address()
    }
}

impl Profile {

    pub fn nomic_claim(&mut self) -> eyre::Result<()> {

        // Create and configure the Command for running "nomic claim"
        let mut cmd = Command::new(CONFIG.nomic()?);
        cmd.arg("claim");

        // Set the environment variables for NOMIC_LEGACY_VERSION
        if let Some(ref version) = CONFIG.nomic_legacy_version {
            cmd.env("NOMIC_LEGACY_VERSION", version);
        }

        // Set the HOME environment variable
        let home_path: &OsStr = self.home().as_os_str();
        cmd.env("HOME", home_path);

        // Execute the command and collect the output
        let output = cmd.output()?;

        // Check if the command was successful
        if !output.status.success() {
            let error_msg = format!(
                "Command `{}` failed with output: {:?}",
                CONFIG.nomic()?,
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
        log: bool,
    ) -> eyre::Result<()> {

        let validator_address = match self.validator_address(validator.as_deref()) {
            Ok(address) => address,
            Err(e) => {
                if log { self.journal().print(Some(OutputFormat::Json))? };
                return Err(eyre!("Failed to resolve validator address: {}", e));
            }
        };

        let quantity_u64 = quantity.map(|n| (n * 1_000_000.0) as u64);

        let calc = self.calc_quantity(validator.as_deref(), quantity_u64);


        if calc.quantity <= 0 {
            if log { self.journal().print(Some(OutputFormat::Json))? };
            return Err(eyre!("Quantity to stake must be greater than 0."));
        }

        if !calc.can_stake_without_claim && !calc.can_stake_after_claim {
            if log { self.journal().print(Some(OutputFormat::Json))? };
            return Err(eyre!("Not enough balance to stake that quantity."));
        }

        if calc.needs_claim {
            if let Err(e) = self.nomic_claim() {
                if log { self.journal().print(Some(OutputFormat::Json))? };
                return Err(eyre!("Failed to claim: {:?}", e));
            }
            self.claimed = true;
//            let balance = self.balance()
//                .saturating_add(*self.total_liquid())
//                .saturating_sub(self.claim_fee());
//            let total_liquid = 0;
//            self.balance = OnceCell::from(balance);
//            self.total_liquid = OnceCell::from(total_liquid);
        }

        // Create and configure the Command for running "nomic delegate"
        let mut cmd = Command::new(CONFIG.nomic()?);

        // Set the environment variables for NOMIC_LEGACY_VERSION
        if let Some(ref version) = CONFIG.nomic_legacy_version {
            cmd.env("NOMIC_LEGACY_VERSION", version);
        }

        // Assuming `self.home()` returns a &Path
        let home_path: &OsStr = self.home().as_os_str();
        cmd.env("HOME", home_path);

        // Add the "delegate" argument, validator, and quantity
        cmd.arg("delegate");
        cmd.arg(validator_address.clone());
        cmd.arg(calc.quantity.to_string());

        // Execute the command and collect the output
        let output = cmd.output()?;

        // Check if the command was successful
        if !output.status.success() {
            let error_msg = format!(
                "Command `{}` failed with output: {:?}",
                CONFIG.nomic()?,
                String::from_utf8_lossy(&output.stderr)
            );
            if log { self.journal().print(Some(OutputFormat::Json))? };
            return Err(eyre!(error_msg));
        }
        self.staked = true;
//        let balance = self.balance()
//            .saturating_sub(calc.quantity)
//            .saturating_sub(self.stake_fee());
//        self.balance = OnceCell::from(balance);
//        let total_staked = self.total_staked()
//            .saturating_add(calc.quantity);
//        self.total_staked = OnceCell::from(total_staked);
//        if self.config().validator_address() == validator_address {
//            let validator_staked = self.validator_staked()
//                .saturating_add(calc.quantity);
//            self.validator_staked = OnceCell::from(validator_staked);
//        }

        // Clone the config
        let mut config = self.config().clone();

        // Rotate the config validators
        config.rotate_validators();
        config.minimum_balance = *self.minimum_balance();
        config.minimum_stake = *self.minimum_stake();
        config.daily_reward = self.daily_reward();
        if let Err(e) = config.save(&self.config_file(), true) {
            warn!("Failed to save config file: {}", e);
        }
        if log { self.journal().print(Some(OutputFormat::Json))? };
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

        // Create and configure the Command for running "nomic delegate"
        let mut cmd = Command::new(CONFIG.nomic()?);

        // Set the environment variables for NOMIC_LEGACY_VERSION
        if let Some(ref version) = CONFIG.nomic_legacy_version {
            cmd.env("NOMIC_LEGACY_VERSION", version);
        }

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
                CONFIG.nomic()?,
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

        // Create and configure the Command for running "nomic delegate"
        let mut cmd = Command::new(CONFIG.nomic()?);

        // Set the environment variables for NOMIC_LEGACY_VERSION
        if let Some(ref version) = CONFIG.nomic_legacy_version {
            cmd.env("NOMIC_LEGACY_VERSION", version);
        }

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
                CONFIG.nomic()?,
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(eyre!(error_msg));
        }

        Ok(())

    }

}

impl Profile {

    pub fn journal(&self) -> &Journal {
        self.journal.get_or_init(|| {
            let mut journal = Journal::new();

            journal.insert(
                "profile".to_string(),
                Value::String(self.name().to_string())
            );
            journal.insert(
                "address".to_string(),
                Value::String(self.address().to_string())
            );
            journal.insert(
                "balance".to_string(),
                Value::Number(self.balance().clone().into())
            );
            journal.insert(
                "total_staked".to_string(),
                Value::Number(self.total_staked().clone().into())
            );
            journal.insert(
                "timestamp".to_string(),
                Value::String(self.delegations_timestamp_rfc3339())
            );
            journal.insert(
                "total_liquid".to_string(),
                Value::Number(self.total_liquid().clone().into())
            );
            journal.insert(
                "config_minimum_balance".to_string(),
                Value::Number(self.config().minimum_balance.into())
            );
            journal.insert(
                "config_minimum_balance_ratio".to_string(),
                Value::Number(self.config().minimum_balance_ratio.into())
            );
            journal.insert(
                "config_minimum_stake".to_string(),
                Value::Number(self.config().minimum_stake.into())
            );
            journal.insert(
                "config_adjust_minimum_stake".to_string(),
                Value::Bool(self.config().adjust_minimum_stake)
            );
            journal.insert(
                "config_minimum_stake_rounding".to_string(),
                Value::Number(self.config().minimum_stake_rounding.into())
            );
            journal.insert(
                "config_daily_reward".to_string(),
                Value::Number(self.config().daily_reward.into())
            );
            journal.insert(
                "config_validator_address".to_string(),
                Value::String(self.config().validator_address().to_string())
            );
            journal.insert(
                "config_validator_name".to_string(),
                Value::String(self.config().validator_name().to_string())
            );
            journal.insert(
                "moniker".to_string(),
                Value::String(self.moniker().to_string())
            );
            journal.insert(
                "voting_power".to_string(),
                Value::Number(self.voting_power().into())
            );
            journal.insert(
                "rank".to_string(),
                Value::Number(self.rank().into())
            );
            journal.insert(
                "validator_staked".to_string(),
                Value::Number(self.validator_staked().clone().into())
            );
            journal.insert(
                "claim_fee".to_string(),
                Value::Number(self.claim_fee().into())
            );
            journal.insert(
                "stake_fee".to_string(),
                Value::Number(self.stake_fee().into())
            );
            journal.insert(
                "minimum_balance".to_string(),
                Value::Number(self.minimum_balance().clone().into())
            );
            journal.insert(
                "minimum_stake".to_string(),
                Value::Number(serde_json::Number::from(*self.minimum_stake()))
            );
            journal.insert(
                "available_without_claim".to_string(),
                Value::Number(serde_json::Number::from(self.calc().available_without_claim))
            );
            journal.insert(
                "available_after_claim".to_string(),
                Value::Number(serde_json::Number::from(self.calc().available_after_claim))
            );
            journal.insert(
                "validator_staked_remainder".to_string(),
                Value::Number(serde_json::Number::from(self.calc().remainder))
            );
            journal.insert(
                "needed".to_string(),
                Value::Number(serde_json::Number::from(self.calc().needed))
            );
            journal.insert(
                "remaining".to_string(),
                Value::Number(serde_json::Number::from(self.calc().remaining))
            );
            journal.insert(
                "can_stake_without_claim".to_string(),
                Value::Bool(self.calc().can_stake_without_claim)
            );
            journal.insert(
                "can_stake_after_claim".to_string(),
                Value::Bool(self.calc().can_stake_after_claim)
            );
            journal.insert(
                "daily_reward".to_string(),
                Value::Number(self.daily_reward().into())
            );
            journal.insert(
                "needs_claim".to_string(),
                Value::Bool(self.calc().needs_claim)
            );
            journal.insert(
                "quantity".to_string(),
                Value::Number(serde_json::Number::from(self.calc().quantity))
            );
            journal.insert(
                "claimed".to_string(),
                Value::String(TaskStatus::from_bool(self.claimed).to_symbol().to_string())
            );
            journal.insert(
                "staked".to_string(),
                Value::String(TaskStatus::from_bool(self.staked).to_symbol().to_string())
            );
            journal
        })
    }

    /// Generates the profile details section with the name and address.
    fn report_profile(&self) -> String {
        let rows = vec![
            TableColumns::new(vec![
                "Profile:",
                self.name(),
            ]),
            TableColumns::new(vec![
                "Address:",
                self.address(),
            ]),
        ];

        let mut builder = Builder::default();
        for row in &rows {
            builder.push_record([row.cell0.clone(), row.cell1.clone()]);
        }

        let mut table = builder.build();
        table
            .with(Style::empty())
            .with(Modify::new(Columns::single(0)).with(Color::new("\x1b[1m", "\x1b[0m")))
            .with(Modify::new(Columns::single(1)).with(Color::FG_GREEN))
            ;

        table.to_string()
    }

    fn report_config(&self) -> String {
        let config = self.config();
        let rows = vec![
            TableColumns::new(vec![ "Configuration:" ]),
            TableColumns::new(vec![
                "Minimum Balance:",
                &NumberDisplay::new(config.minimum_balance).scale(6).decimal_places(6).trim(true).format(),
                "Minimum Balance Ratio:",
                &NumberDisplay::new(config.minimum_balance_ratio).scale(6).decimal_places(6).trim(true).format(),
            ]),
            TableColumns::new(vec![
                "Minimum Stake:",
                &NumberDisplay::new(config.minimum_stake).scale(6).decimal_places(6).trim(true).format(),
            ]),
            TableColumns::new(vec![
                "Adjust Minimum Stake:",
                &config.adjust_minimum_stake.to_string(),
                "Minimum Stake Rounding:",
                &NumberDisplay::new(config.minimum_stake_rounding).scale(6).decimal_places(6).trim(true).format(),
            ]),
        ];

        // Initialize Builder without headers
        let mut builder = Builder::default();
        for row in &rows {
            builder.push_record([row.cell0.clone(), row.cell1.clone(), row.cell2.clone(), row.cell3.clone()]);
        }

        let mut table = builder.build();
        table
            .with(Style::empty())
            .with(Modify::new(Columns::single(1)).with(Border::new().set_right('│')))
            .with(Modify::new(Columns::single(1)).with(Alignment::right()).with(Color::FG_BLUE))
            .with(Modify::new(Columns::single(3)).with(Alignment::right()).with(Color::FG_BLUE))
            .with(Modify::new(Cell::new(0, 0)).with(Span::column(4)))
            .with(Modify::new(Cell::new(0, 0)).with(Color::new("\x1b[1m", "\x1b[0m")))
            ;

        table.to_string()
    }

    fn report_config_validators(&self) -> String {
        let config = self.config();
        let mut rows: Vec<TableColumns> = Vec::new();

        rows.push(
            TableColumns::new(vec![
                "Validators:",
            ]),
        );
        // Iterate over the `validators` in `ConfigValidators`
        for validator in config.validators.iter() {
            rows.push(TableColumns::new(vec![
                &validator.address,
                &validator.name,
            ]));
        }

        // Initialize Builder without headers
        let mut builder = Builder::default();
        for row in &rows {
            builder.push_record([row.cell0.clone(), row.cell1.clone()]);
        }

        let mut table = builder.build();
        table
            .with(Style::empty())
            .with(Modify::new(Rows::single(0)).with(Color::new("\x1b[1m", "\x1b[0m")))
            ;
        for (index, row) in rows.iter().enumerate() {
            if row.cell0 == config.validator_address()  {
                table.with(Modify::new(Rows::single(index)).with(Color::FG_BLUE));
            }
        }

        table.to_string()
    }

    fn get_validator_rank(&self, address: &str) -> String {
        self
            .validators()
            .map_err(|e| {
                warn!("Failed to retrieve validators: {}", e);
                "N/A".to_string()
            })
            .and_then(|validators| {
                validators.validator(address).map_err(|e| {
                    warn!("Validator not found for address {}: {}", address, e);
                    "N/A".to_string()
                })
            })
            .map(|v| v.rank().to_string())
            .unwrap_or_else(|_| "N/A".to_string())
    }

    fn report_delegations(&self) -> String {

        let delegations = match self.delegations() {
            Ok(delegations) => delegations,
            Err(e) => {
                warn!("Failed to retrieve delegations: {}", e);
                return "N/A".to_string();
            }
        };

        // want to convert DateTime<Utc> to DateTime<Local>
        let timestamp_local: DateTime<Local> = delegations.timestamp.with_timezone(&Local);

        let address = self.config().validator_address();

        let mut rows: Vec<TableColumns> = Vec::new();

        rows.push(TableColumns::new(vec![
            //&format!(" \x1b[1m{}\x1b[0m for \x1b[32m{}\x1b[0m as at \x1b[32m{}\x1b[0m",
            &format!("{} for \x1b[32m{}\x1b[0m as at \x1b[32m{}\x1b[0m",
                "Delegations",
                delegations.address,
                timestamp_local.format("%Y-%m-%d %H:%M"),
            ),
        ]));

        rows.push(TableColumns::new(vec![
            "Rank",
            "Validator Address",
            "Moniker",
            "Staked",
            "Liquid",
            "NBTC",
        ]));

        let mut data_rows: Vec<TableColumns> = Vec::new();
        // Iterate over the `delegations` field in `Delegations`
        for (address, delegation) in delegations.delegations.iter() {
        // Attempt to retrieve validators and handle errors if they occur

            data_rows.push(TableColumns::new(vec![
                &self.get_validator_rank(address),
                address,
                self.name_or_moniker(address),
                &NumberDisplay::new(delegation.staked).scale(6).decimal_places(6).trim(true).format(),
                &NumberDisplay::new(delegation.liquid).scale(6).decimal_places(6).format(),
                &NumberDisplay::new(delegation.nbtc).scale(8).decimal_places(8).trim(false).format(),
            ]));
        }

        // Add rows for config validators not yet delegated to
        for validator in self.config().validators.clone() {
            if data_rows.iter().find(|row| row.cell1 == validator.address).is_none() && validator.address != "" {
                data_rows.push(TableColumns::new(vec![
                    &self.get_validator_rank(&validator.address),
                    &validator.address,
                    &validator.name,
                ]));
            }
        };

        // Sort rows descending by `cell0`, converting each `cell0` to `usize`
        data_rows.sort_by(|a, b| {
            a.cell0.parse::<usize>().unwrap_or(0).cmp(&b.cell0.parse::<usize>().unwrap_or(0))
        });

        // Append `data_rows` to `rows`
        rows.extend(data_rows);

        // Create totals row
        rows.push(TableColumns::new(vec![
            "",
            "",
            "Delegations",
            &NumberDisplay::new(delegations.total().staked).scale(6).decimal_places(6).trim(true).format(),
            &NumberDisplay::new(delegations.total().liquid).scale(6).decimal_places(6).trim(false).format(),
            &NumberDisplay::new(delegations.total().nbtc).scale(8).decimal_places(8).trim(false).format(),
        ]));

        // Create balances row
        let nbtc_bal = match self.balances() {
            Ok(b) => b.nbtc,
            Err(_) => 0,
        };
        rows.push(TableColumns::new(vec![
            //&self.report_in(),
            "",
            "",
            "Balances",
            "",
            &NumberDisplay::new(*self.balance()).scale(6).decimal_places(6).trim(false).format(),
            &NumberDisplay::new(nbtc_bal).scale(8).decimal_places(8).trim(false).format(),
        ]));

        // Create totals row
        rows.push(TableColumns::new(vec![
            "",
            //&format!(
            //    "Daily Reward: \x1b[33m{}\x1b[0m", 
            //    NumberDisplay::new(self.daily_reward()).scale(6).decimal_places(6).trim(true).format()
            //),
            "",
            "",
            &NumberDisplay::new(*self.balance() + delegations.total().liquid + delegations.total().staked)
                .scale(6).decimal_places(6).trim(true).integer_threshold(1_000).format(),
            &NumberDisplay::new(*self.balance() + delegations.total().liquid).scale(6).decimal_places(6).trim(false).format(),
            &NumberDisplay::new(nbtc_bal + delegations.total().nbtc).scale(8).decimal_places(8).trim(false).format(),
        ]));

        // Available without claim row
        rows.push(TableColumns::new(vec![
            "",
            "Available without claiming rewards",
            "",
            "",
            &NumberDisplay::new(self.calc().available_without_claim).scale(6).decimal_places(6).trim(false).format(),
        ]));

        // Available after claim row
        rows.push(TableColumns::new(vec![
            "",
            "Available after claiming rewards",
            "",
            "",
            &NumberDisplay::new(self.calc().available_after_claim).scale(6).decimal_places(6).trim(false).format(),
        ]));

        // Initialize Builder without headers
        let mut builder = Builder::default();
        for row in &rows {
            builder.push_record([
                row.cell0.clone(), 
                row.cell1.clone(), 
                row.cell2.clone(), 
                row.cell3.clone(),
                row.cell4.clone(),
                row.cell5.clone(),
            ]);
        }

        let mut table = builder.build();

        table
            .with(Style::blank())
            .with(Modify::new(Columns::single(0)).with(Alignment::right()))
            .with(Modify::new(Columns::new(3..)).with(Alignment::right()))
            // Set borders on the first row
            .with(Modify::new(Cell::new(1, 0)).with(Border::new().set_bottom('-')))
            .with(Modify::new(Cell::new(1, 1)).with(Border::new().set_bottom('-')))
            .with(Modify::new(Cell::new(1, 2)).with(Border::new().set_bottom('-')))
            .with(Modify::new(Cell::new(1, 3)).with(Border::new().set_bottom('-')))
            .with(Modify::new(Cell::new(1, 4)).with(Border::new().set_bottom('-')))
            .with(Modify::new(Cell::new(1, 5)).with(Border::new().set_bottom('-')))
            // Apply right alignment to the totals row (last row)
            .with(Modify::new(Rows::single(rows.len() - 2)).with(Alignment::right()))
            // Apply a distinct bottom border to the totals row
            .with(Modify::new(Cell::new(rows.len() - 5, 3)).with(Border::new().set_top('-')))
            .with(Modify::new(Cell::new(rows.len() - 5, 4)).with(Border::new().set_top('-')))
            .with(Modify::new(Cell::new(rows.len() - 5, 5)).with(Border::new().set_top('-')))
            //.with(Modify::new(Cell::new(rows.len() - 3, 3)).with(Border::new().set_top('=')))
            .with(Modify::new(Cell::new(rows.len() - 3, 4)).with(Border::new().set_top('=')).with(Border::new().set_bottom('-')))
            .with(Modify::new(Cell::new(rows.len() - 3, 5)).with(Border::new().set_top('=')))
            // Total delegation row
            .with(Modify::new(Cell::new(rows.len() - 5, 0)).with(Span::column(2)).with(Alignment::right()))
            .with(Modify::new(Cell::new(rows.len() - 5, 2)).with(Color::FG_GREEN))
            .with(Modify::new(Cell::new(rows.len() - 5, 3)).with(Color::FG_GREEN))
            .with(Modify::new(Cell::new(rows.len() - 5, 4)).with(Color::FG_GREEN))
            .with(Modify::new(Cell::new(rows.len() - 5, 5)).with(Color::FG_GREEN))
            // Balances row
            //.with(Modify::new(Cell::new(rows.len() - 4, 3)).with(Color::FG_GREEN))
            .with(Modify::new(Cell::new(rows.len() - 4, 2)).with(Color::FG_GREEN))
            .with(Modify::new(Cell::new(rows.len() - 4, 3)).with(Color::FG_GREEN))
            .with(Modify::new(Cell::new(rows.len() - 4, 4)).with(Color::FG_GREEN))
            .with(Modify::new(Cell::new(rows.len() - 4, 5)).with(Color::FG_GREEN))
            // Grand totals row
            //.with(Modify::new(Cell::new(rows.len() - 3, 3)).with(Color::new("\x1b[38;2;255;165;0m", "\x1b[0m")))
            .with(Modify::new(Cell::new(rows.len() - 3, 3)).with(Color::new("\x1b[38;2;255;165;0m", "\x1b[0m")))
            .with(Modify::new(Cell::new(rows.len() - 3, 3)).with(Color::FG_YELLOW))
            .with(Modify::new(Cell::new(rows.len() - 3, 4)).with(Color::FG_GREEN))
            .with(Modify::new(Cell::new(rows.len() - 3, 5)).with(Color::FG_GREEN))
            // Apply span to the first two cells in the last row
            .with(Modify::new(Cell::new(0, 0)).with(Span::column(6)).with(Alignment::left()))
            .with(Modify::new(Cell::new(rows.len() - 1, 1)).with(Span::column(3)).with(Alignment::right()))
            .with(Modify::new(Cell::new(rows.len() - 2, 1)).with(Span::column(3)).with(Alignment::right()))
            .with(Modify::new(Cell::new(rows.len() - 1, 4)).with(Color::FG_GREEN))
            .with(Modify::new(Cell::new(rows.len() - 2, 4)).with(Color::FG_GREEN))
            ;

        for (index, row) in rows.iter().enumerate() {
            if row.cell1 == address  {
                table.with(Modify::new(Rows::single(index)).with(Color::FG_BLUE));
            }
        }

        table.to_string()
    }

    fn report_conclusion(&self) -> String {
        // Format daily with yellow text
        let daily_reward = format!(
            "\x1b[33m{}\x1b[0m", 
            NumberDisplay::new(self.daily_reward())
                .scale(6)
                .decimal_places(6)
                .trim(true)
                .format()
        );

        // Format minimum_stake with blue text
        let minimum_stake = format!(
            "\x1b[34m{}\x1b[0m", 
            NumberDisplay::new(*self.minimum_stake())
                .scale(6)
                .decimal_places(6)
                .trim(true)
                .format()
        );

        // Format minimum_period with blue text
        let minimum_period = format!(
            "\x1b[34m{}\x1b[0m",
            format_duration(self.minimum_stake().saturating_mul(86_400).saturating_div(self.daily_reward()))
        );

        let mut output = format!("With an estimated daily reward of {} NOM,", daily_reward);
        output = format!("{}\nIt should take about {}, to earn {} NOM.", output, minimum_period, minimum_stake);


        // Format quantity with blue text
        let quantity = format!(
            "\x1b[34m{}\x1b[0m", 
            NumberDisplay::new(self.calc().quantity)
                .scale(6)
                .decimal_places(6)
                .trim(true)
                .format()
        );

        // Format validator name with blue text
        let validator = format!("\x1b[34m{}\x1b[0m", self.config().validator_name());

        // Handle the case where no staking is needed
        let more = if self.calc().remaining == 0 {
            if self.calc().needs_claim {
                format!("Ready to claim rewards and delegate {} NOM to {}", quantity, validator)
            } else {
                format!("Ready to delegate {} NOM to {}", quantity, validator)
            }
        } else {
            let remaining_seconds = self.calc().remaining.saturating_mul(86_400).saturating_div(self.daily_reward());
            let remaining_period = format!( "\x1b[34m{}\x1b[0m", format_duration(remaining_seconds));
            let remaining_date = format!( "\x1b[34m{}\x1b[0m", format_date_offset(remaining_seconds));
            // Return the formatted string for when staking is needed
            format!(
                "In {} on or about {},\nwe should be ready to claim rewards and delegate {} NOM to {}.",
                remaining_period, remaining_date, quantity, validator
            )
        };
        output = format!("{}\n{}", output, more);
        output

    }

    pub fn report(&self) -> String {
        format!(
            "{}\n\n{}\n\n{}\n\n{}\n{}",
            //"{}\n\n{}\n\n{}\n\n{}\n\n{}",
            //"report_profile",
            self.report_profile(),
            self.report_config(),
            self.report_config_validators(),
            self.report_delegations(),
            //self.report_calc(),
            self.report_conclusion(),
        )
    }
}
