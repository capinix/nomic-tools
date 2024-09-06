use std::env;
// Global constant for the default config directory
static DEFAULT_CONFIG_DIR: &str = "auto/conf.d";
// pub const NOMIC_PATH: &str = "nomic";

pub const MINIMUM_BALANCE: f64 = 0.1;
pub const MINIMUM_BALANCE_RATIO: f64 = 0.001;

pub fn get_default_config_dir() -> String {
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("/home/user")); // Provide a fallback
    format!("{}/{}", home_dir, DEFAULT_CONFIG_DIR)
}

