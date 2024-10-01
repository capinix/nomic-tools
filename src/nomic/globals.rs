use lazy_static::lazy_static;
use std::env;
use std::path::PathBuf;

#[allow(dead_code)]
fn to_bool(val: String) -> Option<bool> {
    match val.trim().to_lowercase().as_str() {
        "true" | "yes" | "y" | "1" => Some(true),
        "false" | "no" | "n" | "0" => Some(false),
        "" => Some(false), // Treat empty string as false
        _ => None, // Invalid value, return None
    }
}

fn to_bool_string(val: String) -> Option<String> {
    match val.trim().to_lowercase().as_str() {
        "true" | "yes" | "y" | "1" => Some("true".to_string()),
        "false" | "no" | "n" | "0" => Some("false".to_string()),
        "" => Some("false".to_string()), // Handle empty string as "false"
        _ => None, // Invalid value, return None
    }
}

lazy_static! {

    pub static ref NOMIC_LEGACY_VERSION: String = env::var("NOMIC_LEGACY_VERSION").ok()
        .unwrap_or(String::new());

    pub static ref MINIMUM_BALANCE: f64 = env::var("MINIMUM_BALANCE").ok()
        .and_then(|val| val.parse::<f64>().ok())
        .unwrap_or(0.1);
    
    pub static ref MINIMUM_BALANCE_RATIO: f64 = env::var("MINIMUM_BALANCE_RATIO").ok()
        .and_then(|val| val.parse::<f64>().ok())
        .unwrap_or(0.001);

    pub static ref MINIMUM_STAKE: f64 = env::var("MINIMUM_STAKE").ok()
        .and_then(|val| val.parse::<f64>().ok())
        .unwrap_or(0.25);

	pub static ref ADJUST_MINIMUM_STAKE: String = {
		to_bool_string(
			env::var("ADJUST_MINIMUM_STAKE").unwrap_or_default() // Safely get the variable value
		).unwrap_or_else(|| "true".to_string()) // Call to_bool_string and provide a default
	};
    
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






