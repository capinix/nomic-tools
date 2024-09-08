use indexmap::IndexMap;
use regex::Regex;
use serde_json::json;
use serde_json::to_string;
// use serde_json::to_string_pretty;
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::nomic::balance;
use crate::nomic::calc;
use crate::nomic::delegations;

pub fn temp_home(profile_path: &Path) -> Result<TempDir, Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let temp_home_path = temp_dir.path().to_path_buf();

    let orga_wallet_dir = temp_home_path.join(".orga-wallet");
    fs::create_dir_all(&orga_wallet_dir)?;

    fs::copy(profile_path, orga_wallet_dir.join("privkey"))?;
    fs::File::create(orga_wallet_dir.join("nonce"))?;

    Ok(temp_dir)
}

pub fn restore_nonce(temp_home_path: &Path, profile_path: &Path) -> Result<(), Box<dyn Error>> {
    let temp_nonce_path = temp_home_path.join(".orga-wallet").join("nonce");
    let profile_nonce_path = profile_path.with_extension("nonce");

    fs::copy(temp_nonce_path, profile_nonce_path)?;

    Ok(())
}

fn change_extension(path: &Path, extension: &str) -> PathBuf {
    let mut new_path = path.to_path_buf();
    new_path.set_extension(extension);
    new_path
}

fn parse_profile_conf(file_path: &Path) -> Result<IndexMap<String, String>, Box<dyn Error>> {
    let file_content = fs::read_to_string(file_path)?; // Propagate the error
    let mut config: IndexMap<String, String> = IndexMap::new();
    let re = Regex::new(r#"^read\s+VALIDATOR\s+MONIKER\s+<<<\s+"([^"]+)"$"#)?;

    for line in file_content.lines() {
        let trimmed_line = line.trim();
        if let Some(caps) = re.captures(trimmed_line) {
            let value = &caps[1];
            let parts: Vec<&str> = value.split_whitespace().collect();
            let validator = parts.get(0).unwrap_or(&"").trim().to_string();
            let moniker = parts.get(1).unwrap_or(&"").trim().to_string();
            config.insert("VALIDATOR".to_string(), validator);
            config.insert("MONIKER".to_string(), moniker);
        } else {
            let mut parts = trimmed_line.split('=');
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();
            config.insert(key.to_string(), value.to_string());
        }
    }

    // Print out the config map for debugging
    // println!("Config: {:?}", config);

    Ok(config)
}


pub fn run_profile(
    home_dir: &str,
    profile_privkey: &Path,
    validator_info: &IndexMap<String, IndexMap<String, String>>, // Changed to a reference
) -> Result<(), Box<dyn Error>> {

    let profile_name = profile_privkey.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
    let home_dir_ref: Option<&str> = Some(home_dir); // Use `Some` directly

    let mut journal = balance::json(home_dir_ref, validator_info)?; // Use `validator_info` as a reference
    journal.insert("profile".to_string(), Value::String(profile_name.to_string()));

	let profile_conf = change_extension(profile_privkey, "conf");
    let mut config_data = parse_profile_conf(profile_conf.as_path())?;

let validator = match config_data.get("VALIDATOR") {
    Some(value) => value.to_string(),
    None => {
        eprintln!("Missing 'config:VALIDATOR' key");
        return Ok(()); // Adjust this return based on your function's actual return type
    }
};

	let voting_power = validator_info.get(&validator)
		.and_then(|info| info.get("VOTING POWER"))
		.cloned()
		.unwrap_or("".to_string()).trim()
		.parse::<u64>().unwrap_or_default();

	let moniker = validator_info.get(&validator)
		.and_then(|info| info.get("MONIKER"))
		.cloned()
		.unwrap_or_default();

	config_data.insert("moniker".to_string(), moniker);
	config_data.insert("voting_power".to_string(), voting_power.to_string());

// 	let voting_power = validator_get(

    journal.insert("config".to_string(), json!(config_data));

    // Serialize journal to a string before parsing
    let journal_string = serde_json::to_string(&journal)?;

    // Parse journal_string back to Value
    let v: Value = serde_json::from_str(&journal_string)?;
    
    let delegations = v.get("delegations").ok_or("Missing 'delegations' key")?;

    let totals = delegations::totals(delegations);
    journal.insert("total".to_string(), totals);

	let stake_calc = calc::stake(&journal);
    journal.insert("calc".to_string(), stake_calc);


    // Serialize and print the JSON
//	let json_string = to_string_pretty(&journal)?;
    let json_string = to_string(&journal)?;
    println!("{}", json_string);

    Ok(())
}
