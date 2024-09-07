mod nomic {
    pub mod validators;
    pub mod balance;
    pub mod delegations;
}

use std::env;
use nomic::{balance, validators};
// use serde_json::to_string_pretty;
use serde_json::to_string;
// use serde_json::Value;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let home_dir = env::var("HOME").ok();
    let validator_info = validators::json()?;

    // Call the json function
    let json_result = balance::json(home_dir.as_deref(), validator_info)?;

    // Serialize and print the JSON
    let json_string = to_string(&json_result)?;
    println!("{}", json_string);

    Ok(())
}
