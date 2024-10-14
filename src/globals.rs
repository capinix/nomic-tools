use lazy_static::lazy_static;
use std::env;
use std::path::PathBuf;
use crate::functions::to_bool_string;

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
