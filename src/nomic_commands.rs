use std::process::Command;
use indexmap::IndexMap;
use serde_json::json;
use chrono::Local;
use tempfile::TempDir;
use std::path::Path;
use std::fs;

use crate::calc;
use crate::globals::*;

use crate::commands::{parse_profile_config, change_extension, get_last_journal};

pub fn temp_home(profile_path: &Path) -> Result<TempDir, Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let temp_home_path = temp_dir.path().to_path_buf();

    let orga_wallet_dir = temp_home_path.join(".orga-wallet");
    fs::create_dir_all(&orga_wallet_dir)?;

    fs::copy(profile_path, orga_wallet_dir.join("privkey"))?;
    fs::File::create(orga_wallet_dir.join("nonce"))?;

    Ok(temp_dir)
}

pub fn restore_nonce(temp_home_path: &Path, profile_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let temp_nonce_path = temp_home_path.join(".orga-wallet").join("nonce");
    let profile_nonce_path = profile_path.with_extension("nonce");

    fs::copy(temp_nonce_path, profile_nonce_path)?;

    Ok(())
}

/// Function to clean a field by removing control characters
fn clean_field(field: &str) -> String {
    // Remove non-graphic characters and whitespace
    let cleaned = field.chars()
                       .filter(|&c| c.is_ascii_graphic() || c.is_whitespace())
                       .collect::<String>()
                       .trim() // Trim leading and trailing whitespace
                       .to_string(); // Convert back to String
                       
    cleaned
}

pub fn validators() -> Result<IndexMap<String, (String, u64, usize)>, Box<dyn std::error::Error>> {
    let mut validators_command = Command::new("nomic");
    validators_command.arg("validators");
// 	validators_command.env("HOME", home_dir);
    let validators_output = validators_command.output()?;

    if !validators_output.status.success() {
        return Err(format!("nomic validators command failed with output: {:?}", validators_output).into());
    }

    let validators_output_str = String::from_utf8_lossy(&validators_output.stdout);
    let validators_lines: Vec<&str> = validators_output_str.lines().collect();

//     // Debugging: Print the output of `nomic validators`
//     println!("nomic validators output:");
//     println!("{}", validators_output_str);


    let mut validator_info = IndexMap::new();
    let mut rank = 1; // Start rank from 1

    for chunk in validators_lines.chunks(4) {
        if chunk.len() == 4 {
            let address = chunk[0].trim().trim_start_matches('-').trim().to_string();
            let voting_power_str = chunk[1].split(':').nth(1).unwrap_or("").trim();
            let moniker_temp = chunk[2].split(':').nth(1).unwrap_or("").trim().to_string();
            let moniker = clean_field(&moniker_temp);

//             // Debugging: Print parsed values before inserting
//             println!("Parsed Validator Address: {}", address);
//             println!("Parsed Voting Power: {}", voting_power_str);
//             println!("Parsed Moniker: {}", moniker);


            if let Ok(voting_power) = voting_power_str.parse::<u64>() {
                validator_info.insert(address, (moniker, voting_power, rank));
                rank += 1;
            }
        }
    }

    // Debugging: Print the entire validator_info map

    Ok(validator_info)
}

pub fn balance(home_dir: &str) -> Result<(String, IndexMap<String, String>), Box<dyn std::error::Error>> {
    let mut balance_command = Command::new("nomic");
    balance_command.arg("balance");
	balance_command.env("HOME", home_dir);
    let balance_output = balance_command.output()?;

    if !balance_output.status.success() {
        return Err(format!("nomic balance command failed with output: {:?}", balance_output).into());
    }

    let balance_output_str = String::from_utf8_lossy(&balance_output.stdout);
    let balance_lines: Vec<&str> = balance_output_str.lines().collect();

    let address_line = balance_lines.get(0).unwrap_or(&"");
    let address = address_line.split_whitespace().nth(1).unwrap_or("").to_string();

    let mut nb_values = IndexMap::new();
    for line in &balance_lines[1..] {
        let mut parts = line.split_whitespace();
        let value = parts.next().unwrap_or("");
        let label = parts.collect::<Vec<&str>>().join(" ");
        nb_values.insert(label, value.to_string());
    }

    Ok((address, nb_values))
}

pub fn delegations(
	home_dir: &str, 
	validator_info: &IndexMap<String, (String, u64, usize)>
) -> Result<(u64, u64, u64, Vec<IndexMap<String, String>>), Box<dyn std::error::Error>> {

    let mut delegations_command = Command::new("nomic");
    delegations_command.arg("delegations");
	delegations_command.env("HOME", home_dir);
    let delegations_output = delegations_command.output()?;

    if !delegations_output.status.success() {
        return Err(format!("nomic delegations command failed with output: {:?}", delegations_output).into());
    }

    let delegations_output_str = String::from_utf8_lossy(&delegations_output.stdout);
    let delegations_lines: Vec<&str> = delegations_output_str.lines().collect();

    let mut total_staked = 0;
    let mut total_liquid = 0;
    let mut total_nbtc = 0;
    let mut delegations_data = Vec::new();

    for line in &delegations_lines[1..] {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let mut delegation = IndexMap::new();

            let validator_address = parts[1].trim_end_matches(':').to_string();
            delegation.insert("validator".to_string(), validator_address.clone());

            let staked_str = parts[2].trim_start_matches("staked=");
            if let Ok(staked) = staked_str.trim().parse::<u64>() {
                total_staked += staked;
                delegation.insert("staked".to_string(), staked.to_string());
            }

            let liquid_str = parts[4].trim_start_matches("liquid=");
            if let Ok(liquid) = liquid_str.trim().parse::<u64>() {
                total_liquid += liquid;
                delegation.insert("liquid".to_string(), liquid.to_string());
            }

            let nbtc_str = parts[5].trim_start_matches("NOM,");
            if let Ok(nbtc) = nbtc_str.trim().parse::<u64>() {
                total_nbtc += nbtc;
                delegation.insert("nbtc".to_string(), nbtc.to_string());
            }

            // Add Moniker, Voting Power, and Rank from validator_info
            if let Some((moniker, voting_power, rank)) = validator_info.get(&validator_address) {
                delegation.insert("moniker".to_string(), moniker.clone());
                delegation.insert("voting_power".to_string(), voting_power.to_string());
                delegation.insert("rank".to_string(), rank.to_string());
            }

            delegation.insert("nbtc".to_string(), "0".to_string()); // Assuming nbtc is always "0"

            delegations_data.push(delegation);
        }
    }

    Ok((total_staked, total_liquid, total_nbtc, delegations_data))
}

pub fn format_json_output(
    timestamp: u64,
    profile_name: &str,
    address: &str,
    nb_values: IndexMap<String, String>,
    total_staked: u64,
    total_liquid: u64,
    total_nbtc: u64,
    daily_reward: u64,
    delegations_data: Vec<IndexMap<String, String>>, // Ensure this matches what `delegations` returns
    config_data: IndexMap<String, String>,
) -> String {

    let mut json_data: IndexMap<String, serde_json::Value> = IndexMap::new();
    json_data.insert("timestamp".to_string(), json!(timestamp.to_string()));
    json_data.insert("profile".to_string(), json!(profile_name));
    json_data.insert("address".to_string(), json!(address));

    for (key, value) in nb_values {
        json_data.insert(key, json!(value));
    }

    json_data.insert("total_staked".to_string(), json!(total_staked.to_string()));
    json_data.insert("total_liquid".to_string(), json!(total_liquid.to_string()));
    json_data.insert("total_nbtc".to_string(), json!(total_nbtc.to_string()));
    json_data.insert("daily_reward".to_string(), json!(daily_reward.to_string()));
    json_data.insert("delegations".to_string(), serde_json::to_value(delegations_data).unwrap());
    json_data.insert("config".to_string(), serde_json::to_value(config_data).unwrap());

    serde_json::to_string(&json_data).unwrap()
}

pub fn run_commands(
	home_dir: &str,
	profile_path: &Path,
    validator_info: &IndexMap<String, (String, u64, usize)>, // Lookup table for validators
) -> Result<(), Box<dyn std::error::Error>> { // Adjusted return type

	let profile_name = profile_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

    // Fetch the last journal entry as a JSON Value
    let last_journal = get_last_journal(&profile_name)?;

// 	println!("last journal: {}", last_journal);

    // Extract values as strings and then parse them into appropriate types
    let last_timestamp = last_journal["timestamp"]
        .as_str()
        .unwrap_or_default()
        .parse::<u64>()
        .unwrap_or(0);

    let last_total_staked = last_journal["total_staked"]
        .as_str()
        .unwrap_or_default()
        .parse::<u64>()
        .unwrap_or(0);

    let last_total_liquid = last_journal["total_liquid"]
        .as_str()
        .unwrap_or_default()
        .parse::<u64>()
        .unwrap_or(0);

    // Get the current timestamp as a u64
    let timestamp = Local::now().timestamp() as u64;
	
    // Fetch balance data using the balance function from nomic_commands
    let (address, nb_values) = balance(&home_dir)?;

	let balance = nb_values.get("NOM")
		.and_then(|value| value.parse::<u64>().ok())
		.unwrap_or(0);

// println!("balance: {}", balance);

    // Fetch delegations data using the delegations function from nomic_commands
    let (total_staked, total_liquid, total_nbtc, delegations_data) = delegations(&home_dir, validator_info)?;

	let config_path = change_extension(profile_path, "conf");
	let config_data = parse_profile_config(config_path.as_path());

	let minimum_balance_config_nom = config_data.get("MINIMUM_BALANCE")
		.and_then(|value| value.parse::<f64>().ok())
		.unwrap_or(*MINIMUM_BALANCE);

	let minimum_balance_ratio = config_data.get("MINIMUM_BALANCE_RATIO")
		.and_then(|value| value.parse::<f64>().ok())
		.unwrap_or(*MINIMUM_BALANCE_RATIO);
	let minimum_balance_config = ( minimum_balance_config_nom as f64 * 1_000_000.0 ) as u64;

	let minimum_balance = calc::minimum_balance(
        total_staked,
        minimum_balance_ratio,
        minimum_balance_config,
	);

	let minimum_stake_config_nom = config_data.get("MINIMUM_STAKE")
		.and_then(|value| value.parse::<f64>().ok())
		.unwrap_or(*MINIMUM_STAKE);
	let minimum_stake_config = ( minimum_stake_config_nom as f64 * 1_000_000.0 ) as u64;

	let minimum_stake_rounding = config_data.get("MINIMUM_STAKE_ROUNDING")
		.and_then(|value| value.parse::<f64>().ok())
		.unwrap_or(*MINIMUM_STAKE_ROUNDING);

	let validator = config_data.get("VALIDATOR")
		.and_then(|value| value.parse::<f64>().ok())
		.unwrap_or(*VALIDATOR);

// println!("minimum_balance: {}", minimum_balance);
	 
	let staked_amount = delegations_data["delegations"]
			.as_array()
			.and_then(|delegations| {
				delegations.iter().find_map(|delegation| {
					if delegation.get("validator")?.as_str()? == validator {
						delegation.get("staked")?.as_str()?.parse::<u64>().ok()
					} else {
						None
					}
				})
			});

		match staked_amount {
			Some(amount) => println!("Staked amount for validator {} is {}", validator, amount),
			None => println!("Validator {} not found or staked amount is missing.", validator),
		}
	}

// println!("minimum_balance: {}", minimum_balance);

    let daily_reward = calc::daily_reward(
        timestamp,
        total_staked,
        total_liquid,
        last_timestamp,
        last_total_staked,
        last_total_liquid,
    );

	let should_claim, stake_quantity = calc::stake(
		total_liquid,
		balance,
		*CLAIM_FEE,
		*STAKE_FEE,
		minimum_balance,
		staked_amount,
		minimum_stake,
		adjust,
		daily_reward,
		minimum_stake_rounding,
	)

// println!("timestamp: {}", timestamp);
// println!("last_timestamp: {}", last_timestamp);
// println!("total_staked: {}", total_staked);
// println!("last_total_staked: {}", last_total_staked);
// println!("total_liquid: {}", total_liquid);
// println!("last_total_liquid: {}", last_total_liquid);
// println!("daily_reward: {}", daily_reward);

//     println!("Daily Reward: {}", daily_reward);


	let json_output = format_json_output(
		timestamp,
		&profile_name,
		&address,
		nb_values,
		total_staked,
		total_liquid,
		total_nbtc,
		daily_reward,
		delegations_data,
		config_data,
	);

	println!("{}", json_output);

    Ok(())
}

