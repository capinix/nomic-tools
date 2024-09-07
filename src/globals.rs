use lazy_static::lazy_static;
use std::env;
// Global constant for the default config directory
static DEFAULT_CONFIG_DIR: &str = "auto/conf.d";
pub fn get_default_config_dir() -> String {
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("/home/user")); // Provide a fallback
    format!("{}/{}", home_dir, DEFAULT_CONFIG_DIR)
}

lazy_static! {
    // Default values are provided as fallbacks
    pub static ref MINIMUM_BALANCE: f64 = env::var("MINIMUM_BALANCE")
        .ok()
        .and_then(|val| val.parse::<f64>().ok())
        .unwrap_or(0.1);
    
    pub static ref MINIMUM_BALANCE_RATIO: f64 = env::var("MINIMUM_BALANCE_RATIO")
        .ok()
        .and_then(|val| val.parse::<f64>().ok())
        .unwrap_or(0.001);

    pub static ref MINIMUM_STAKE: f64 = env::var("MINIMUM_STAKE")
        .ok()
        .and_then(|val| val.parse::<f64>().ok())
        .unwrap_or(0.25);

	pub static ref ADJUST_MINIMUM_STAKE: String = env::var("ADJUST_MINIMUM_STAKE")
		.unwrap_or_else(|_| "true".to_string());
    
    pub static ref MINIMUM_STAKE_ROUNDING: f64 = env::var("MINIMUM_STAKE_ROUNDING")
        .ok()
        .and_then(|val| val.parse::<f64>().ok())
        .unwrap_or(0.25);
    
    pub static ref CLAIM_FEE: f64 = env::var("CLAIM_FEE")
        .ok()
        .and_then(|val| val.parse::<f64>().ok())
        .unwrap_or(0.01);
    
    pub static ref STAKE_FEE: f64 = env::var("STAKE_FEE")
        .ok()
        .and_then(|val| val.parse::<f64>().ok())
        .unwrap_or(0.01);
}


// pub const NOMIC_PATH: &str = "nomic";






