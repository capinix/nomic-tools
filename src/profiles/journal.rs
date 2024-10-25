use once_cell::unsync::OnceCell;
use crate::profiles::Profile;

use indexmap::IndexMap;
use serde_json::{self, Value};
use serde::{Deserialize, Serialize};





// Trait for getting values of different types
trait GetValue<T> {
    fn get(&self, key: &str) -> Option<T>;
}

// Implement the trait for YourStruct
impl GetValue<u64> for YourStruct {
    fn get(&self, key: &str) -> Option<u64> {
        self.data.get(key).and_then(|value| value.as_u64())
    }
}

impl GetValue<String> for YourStruct {
    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).and_then(|value| value.as_str().map(|s| s.to_string()))
    }
}

impl GetValue<DateTime<Utc>> for YourStruct {
    fn get(&self, key: &str) -> Option<DateTime<Utc>> {
        self.data.get(key).and_then(|value| {
            value.as_str()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
        })
    }
}

impl GetValue<f64> for YourStruct {
    fn get(&self, key: &str) -> Option<f64> {
        self.data.get(key).and_then(|value| value.as_f64())
    }
}

















#[derive(Serialize, Deserialize, Debug)]
pub struct Journal(IndexMap<String, Value>);

impl Journal {
    // Convert IndexMap to JSON
    pub fn to_json(data: &IndexMap<String, Value>) -> JsonResult<String> {
        serde_json::to_string(data)
    }

    // Create IndexMap from JSON
    pub fn from_json(json_str: &str) -> JsonResult<IndexMap<String, Value>> {
        serde_json::from_str(json_str)
    }
}

#[derive(Clone)]
pub struct Stats<'a> {
    pub  profile:                        Option<&'a Profile<'a>>,
    pub  name:                           OnceCell<String>,
    pub  address:                        OnceCell<String>,
    pub  balance:                        OnceCell<u64>,
    pub  total_staked:                   OnceCell<u64>,
    pub  timestamp:                      OnceCell<String>,
    pub  total_liquid:                   OnceCell<u64>,
    pub  config_minimum_balance:         OnceCell<u64>,
    pub  config_minimum_balance_ratio:   OnceCell<f64>,
    pub  config_minimum_stake:           OnceCell<u64>,
    pub  config_adjust_minimum_stake:    OnceCell<bool>,
    pub  config_minimum_stake_rounding:  OnceCell<u64>,
    pub  config_daily_reward:            OnceCell<f64>,
    pub  config_validator:               OnceCell<String>,
    pub  config_moniker:                 OnceCell<String>,
    pub  moniker:                        OnceCell<String>,
    pub  voting_power:                   OnceCell<u64>,
    pub  rank:                           OnceCell<u64>,
    pub  validator_staked:               OnceCell<u64>,
    pub  claim_fee:                      OnceCell<u64>,
    pub  stake_fee:                      OnceCell<u64>,
    pub  minimum_balance:                OnceCell<u64>,
    pub  stake_factor:                   OnceCell<u64>,
    pub  available_before_claim:         OnceCell<u64>,
    pub  available_after_claim:          OnceCell<u64>,
    pub  validator_staked_remainder:     OnceCell<u64>,
    pub  can_stake_before_claim:         OnceCell<bool>,
    pub  can_stake_after_claim:          OnceCell<bool>,
    pub  daily_reward:                   OnceCell<f64>,
    pub  claim:                          OnceCell<bool>,
    pub  quantity_to_stake:              OnceCell<u64>,
}

impl<'a> Stats<'a> {
    pub fn new(
        home:        PathBuf,
        key:         PrivKey,
        config:      Config,
        balances:    Balance,
        delegations: Delegations,
        validators:  ValidatorCollection,
    ) -> Self {
        Stats {
            profile,
            name:                           OnceCell::new(),
            address:                        OnceCell::new(),
            balance:                        OnceCell::new(),
            total_staked:                   OnceCell::new(),
            timestamp:                      OnceCell::new(),
            total_liquid:                   OnceCell::new(),
            config_minimum_balance:         OnceCell::new(),
            config_minimum_balance_ratio:   OnceCell::new(),
            config_minimum_stake:           OnceCell::new(),
            config_adjust_minimum_stake:    OnceCell::new(),
            config_minimum_stake_rounding:  OnceCell::new(),
            config_daily_reward:            OnceCell::new(),
            config_validator:               OnceCell::new(),
            config_moniker:                 OnceCell::new(),
            moniker:                        OnceCell::new(),
            voting_power:                   OnceCell::new(),
            rank:                           OnceCell::new(),
            validator_staked:               OnceCell::new(),
            claim_fee:                      OnceCell::new(),
            stake_fee:                      OnceCell::new(),
            minimum_balance:                OnceCell::new(),
            stake_factor:                   OnceCell::new(),
            available_before_claim:         OnceCell::new(),
            available_after_claim:          OnceCell::new(),
            validator_staked_remainder:     OnceCell::new(),
            can_stake_before_claim:         OnceCell::new(),
            can_stake_after_claim:          OnceCell::new(),
            daily_reward:                   OnceCell::new(),
            claim:                          OnceCell::new(),
            quantity_to_stake:              OnceCell::new(),
        }
    }
}

impl<'a> Default for Stats<'a> {
    fn default() -> Self {
        Self::new(None)
    }
}

impl<'a> Stats<'a> {
    // Methods to retrieve values, utilizing OnceCell
    pub fn name(&self) -> String {
        self.name.get_or_try_init(|| {
            if let Some(profile) = self.profile {
                match profile.name() {
                    Ok(name) => Ok(name.to_string()), // Convert to String
                    Err(err) => {
                        eprintln!("Failed to get profile name: {}", &err);
                        Err(eyre::eyre!("Failed to retrieve name")) // Return an error
                    }
                }
            } else {
                // Handle the case where profile is None
                Err(eyre::eyre!("Profile is not available"))
            }
        }).map(|s| s.clone()) // Clone the String to match the expected return type
        .unwrap_or_else(|_| "".to_string()) // Return an empty String on error
    }

    pub fn address(&self) -> String {
        self.address.get_or_try_init(|| {
            if let Some(profile) = self.profile {
                match profile.address() {
                    Ok(address) => Ok(address.to_string()), // Convert to String
                    Err(err) => {
                        eprintln!("Failed to get profile address: {}", &err);
                        Err(eyre::eyre!("Failed to retrieve address")) // Return an error
                    }
                }
            } else {
                // Handle the case where profile is None
                Err(eyre::eyre!("Profile is not available"))
            }
        }).map(|s| s.clone()) // Clone the String to match the expected return type
        .unwrap_or_else(|_| "".to_string()) // Return an empty String on error
    }

    pub fn balance(&self) -> u64 {
        match self.balance.get_or_init(|| {
            // Use `eyre::Result` for the inner closure to handle errors.
            let result: eyre::Result<u64> = self.profile.balances().map(|b| b.nom);
            result // This returns an `eyre::Result<u64>`, which can be matched on
        }) {
            Ok(&b) => b,  // Return the balance if successful
            Err(err) => {
                eprintln!("Failed to get balance: {}", eyre::Report::new(err));
                0 // Return a default value (e.g., 0) in case of error
            }
        }
    }

    pub fn total_staked(&self) -> u64 {
        *self.total_staked.get_or_init(|| self.profile.total_staked().unwrap_or_else(|_| 0))
    }

    pub fn timestamp(&self) -> &str {
        self.timestamp.get_or_init(|| {
            self.profile.timestamp()
                .map_or_else(|_| "unknown".to_string(), |t| t.to_rfc3339())
        })
    }

    pub fn total_liquid(&self) -> u64 {
        *self.total_liquid.get_or_init(|| self.profile.total_liquid().unwrap_or_else(|_| 0))
    }

    pub fn config_minimum_balance(&self) -> u64 {
        *self.config_minimum_balance.get_or_init(|| self.profile.config_minimum_balance().unwrap_or_else(|_| 0))
    }

    pub fn config_minimum_balance_ratio(&self) -> f64 {
        *self.config_minimum_balance_ratio.get_or_init(|| self.profile.config_minimum_balance_ratio().unwrap_or_else(|_| 0.0))
    }

    pub fn config_minimum_stake(&self) -> u64 {
        *self.config_minimum_stake.get_or_init(|| self.profile.config_minimum_stake().unwrap_or_else(|_| 0))
    }

    pub fn config_adjust_minimum_stake(&self) -> bool {
        *self.config_adjust_minimum_stake.get_or_init(|| self.profile.config_adjust_minimum_stake().unwrap_or_else(|_| false))
    }

    pub fn config_minimum_stake_rounding(&self) -> u64 {
        *self.config_minimum_stake_rounding.get_or_init(|| self.profile.config_minimum_stake_rounding().unwrap_or_else(|_| 0))
    }

    pub fn config_daily_reward(&self) -> f64 {
        *self.config_daily_reward.get_or_init(|| self.profile.config_daily_reward().unwrap_or_else(|_| 0.0))
    }

    pub fn config_validator(&self) -> &str {
        self.config_validator.get_or_init(|| self.profile.config_validator().unwrap_or_default())
    }

    pub fn config_moniker(&self) -> &str {
        self.config_moniker.get_or_init(|| self.profile.config_moniker().unwrap_or_default())
    }

    pub fn moniker(&self) -> &str {
        self.moniker.get_or_init(|| self.profile.moniker().unwrap_or_default())
    }

    pub fn voting_power(&self) -> u64 {
        *self.voting_power.get_or_init(|| self.profile.voting_power().unwrap_or_else(|_| 0))
    }

    pub fn rank(&self) -> u64 {
        *self.rank.get_or_init(|| self.profile.rank().unwrap_or_else(|_| 0))
    }

    pub fn validator_staked(&self) -> u64 {
        *self.validator_staked.get_or_init(|| self.profile.validator_staked().unwrap_or_else(|_| 0))
    }

    pub fn claim_fee(&self) -> u64 {
        *self.claim_fee.get_or_init(|| self.profile.claim_fee().unwrap_or_else(|_| 0))
    }

    pub fn stake_fee(&self) -> u64 {
        *self.stake_fee.get_or_init(|| self.profile.stake_fee().unwrap_or_else(|_| 0))
    }

    pub fn minimum_balance(&self) -> u64 {
        *self.minimum_balance.get_or_init(|| self.profile.minimum_balance().unwrap_or_else(|_| 0))
    }

    pub fn stake_factor(&self) -> u64 {
        *self.stake_factor.get_or_init(|| self.profile.stake_factor().unwrap_or_else(|_| 0))
    }

    pub fn available_before_claim(&self) -> u64 {
        *self.available_before_claim.get_or_init(|| self.profile.available_before_claim().unwrap_or_else(|_| 0))
    }

    pub fn available_after_claim(&self) -> u64 {
        *self.available_after_claim.get_or_init(|| self.profile.available_after_claim().unwrap_or_else(|_| 0))
    }

    pub fn validator_staked_remainder(&self) -> u64 {
        *self.validator_staked_remainder.get_or_init(|| self.profile.validator_staked_remainder().unwrap_or_else(|_| 0))
    }

    pub fn can_stake_before_claim(&self) -> bool {
        *self.can_stake_before_claim.get_or_init(|| self.profile.can_stake_before_claim().unwrap_or_else(|_| false))
    }

    pub fn can_stake_after_claim(&self) -> bool {
        *self.can_stake_after_claim.get_or_init(|| self.profile.can_stake_after_claim().unwrap_or_else(|_| false))
    }

    pub fn daily_reward(&self) -> f64 {
        *self.daily_reward.get_or_init(|| self.profile.config_daily_reward().unwrap_or_else(|_| 0.0))
    }

    pub fn claim(&self) -> bool {
        *self.claim.get_or_init(|| self.profile.claim().unwrap_or_else(|_| false))
    }

    pub fn quantity_to_stake(&self) -> u64 {
        *self.quantity_to_stake.get_or_init(|| self.profile.quantity_to_stake().unwrap_or_else(|_| 0))
    }
}
