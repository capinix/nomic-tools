use serde_json::json;
use serde_json::Value;
use serde_json::Map;
use std::error::Error;
use std::process::Command;
use core::cmp::max;
use crate::globals::{MINIMUM_BALANCE, MINIMUM_BALANCE_RATIO};
use crate::globals::{MINIMUM_STAKE, ADJUST_MINIMUM_STAKE, MINIMUM_STAKE_ROUNDING};
use crate::globals::{CLAIM_FEE, STAKE_FEE};

pub fn get_last_journal(address: &str) -> Result<Value, Box<dyn Error>> {
	// Prepare the grep expression, escaping necessary characters
	let grep_expr = format!(r#"{{.*"address"[[:space:]]*:[[:space:]]*"{}".*}}"#, address);

	// Build the command to execute
	let output = Command::new("journalctl")
		.args(&[
			"-u", "nomic-status",
			"-r",
			"-o", "cat",
			"--no-pager",
			"--lines", "1",
			"-g", &grep_expr,
		])
		.output()?;

	// Check if the command executed successfully and has output
	if !output.status.success() || output.stdout.is_empty() {
		// Return default JSON object if command fails or returns no output
		return Ok(json!({
			"timestamp": "0",
			"total": {
				"liquid": {
					"NBTC": "0",
					"NOM": "0"
				},
				"staked": "0"
			}
		}));
	}

	// Convert the output to a string
	let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

	// Parse the string as JSON
	let json: Value = serde_json::from_str(&output_str).unwrap_or_else(|_| {
		// Return default JSON object if parsing fails
		json!({
			"timestamp": "0",
			"total": {
				"liquid": {
					"NBTC": "0",
					"NOM": "0"
				},
				"staked": "0"
			}
		})
	});

	Ok(json)
}

fn calculate_minimum_balance(
    total_staked: u64,
    balance_ratio: f64,
    current_minimum_balance: u64,
) -> u64 {
	let minimum_balance = (total_staked as f64 * balance_ratio).round() as u64;
	max(minimum_balance, current_minimum_balance)
}

pub fn minimum_balance(journal: &Map<String, Value>) -> u64 {

    let config_key = match journal.get("config") {
        Some(config) => config,
        None => {
            println!("Missing 'config' key");
            return 0;
        }
    };

    let config_minimum_balance = match config_key.get("MINIMUM_BALANCE")
        .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())) {
        Some(value) => (value * 1_000_000.0) as u64,
        None => {
            println!("Missing or invalid 'config:MINIMUM_BALANCE' key");
            println!("Using global default: {}", *MINIMUM_BALANCE);
            (*MINIMUM_BALANCE * 1_000_000.0) as u64
        }
    };

    let config_minimum_balance_ratio = match config_key.get("MINIMUM_BALANCE_RATIO")
        .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())) {
        Some(value) => value,
        None => {
            println!("Missing or invalid 'config:MINIMUM_BALANCE_RATIO' key");
            println!("Using global default: {}", *MINIMUM_BALANCE_RATIO);
            *MINIMUM_BALANCE_RATIO
        }
    };

    let total_key = match journal.get("total") {
        Some(total) => total,
        None => {
            println!("Missing 'total' key");
            return config_minimum_balance;
        }
    };

    let total_staked = match total_key.get("staked")
        .and_then(|v| v.as_str().and_then(|s| s.parse::<u64>().ok())) {
        Some(value) => value,
        None => {
            println!("Missing or invalid 'total:staked' key");
            return config_minimum_balance;
        }
    };

	calculate_minimum_balance(
		total_staked,
		config_minimum_balance_ratio,
		config_minimum_balance,
	)
}

fn calculate_daily_reward(
    current_timestamp: u64,
    current_total_staked: u64,
    current_total_liquid: u64,
    last_timestamp: u64,
    last_total_staked: u64,
    last_total_liquid: u64,
) -> u64 {
    // Check if the conditions are met
    if last_total_staked == current_total_staked
        && last_total_liquid < current_total_liquid
        && last_timestamp < current_timestamp
        && last_timestamp > 0
    {
        // Calculate the deltas
        let reward_delta = current_total_liquid - last_total_liquid;
        let time_delta = current_timestamp - last_timestamp;
        
        // Calculate the daily reward as an integer
        let daily_reward = (reward_delta as f64 / time_delta as f64 * 86400.0).round() as u64;

        daily_reward
    } else {
        0
    }
}


pub fn daily_reward(journal: &Map<String, Value>) -> u64 {

	let address = match journal.get("address") {
		Some(value) => value.as_str().unwrap_or(""),
		None => {
			println!("Missing 'address' key");
			return 0;
		}
	};

	let timestamp = match journal.get("timestamp")
		.and_then(|v| v.as_str()
		.and_then(|s| s.parse::<u64>().ok())) {
		Some(value) => value,
		None => {
			println!("Missing or invalid 'timestamp' key");
			return 0;
		}
	};

	let total_key = match journal.get("total") {
		Some(value) => value,
		None => {
			println!("Missing or invalid 'total' key");
			return 0;
		}
	};

	let total_staked = match total_key.get("staked")
		.and_then(Value::as_str)
		.and_then(|s| s.parse::<u64>().ok()) {
		Some(value) => value,
		None => {
			println!("Missing or invalid 'total:staked' key");
			return 0;
		}
	};

	let total_liquid_key = match total_key.get("liquid") {
		Some(value) => value,
		None => {
			println!("Missing or invalid 'total:liquid' key");
			return 0;
		}
	};

	let total_liquid = match total_liquid_key.get("NOM")
		.and_then(Value::as_str)
		.and_then(|s| s.parse::<u64>().ok()) {
		Some(value) => value,
		None => {
			println!("Missing or invalid 'total:liquid:NOM' key");
			return 0;
		}
	};

	let last_journal = match get_last_journal(&address) {
		Ok(journal) => journal,
		Err(e) => {
			println!("Error retrieving last_journal: {}", e);
			return 0;
		}
	};

	let last_timestamp = match last_journal.get("timestamp")
		.and_then(|v| v.as_str()
		.and_then(|s| s.parse::<u64>().ok())) {
		Some(value) => value,
		None => {
			println!("Missing or invalid last journal 'timestamp' key");
			return 0;
		}
	};

	let last_total_key = match last_journal.get("total") {
		Some(value) => value,
		None => {
			println!("Missing or invalid last journal 'total' key");
			return 0;
		}
	};

	let last_total_staked = match last_total_key.get("staked")
		.and_then(Value::as_str)
		.and_then(|s| s.parse::<u64>().ok()) {
		Some(value) => value,
		None => {
			println!("Missing or invalid last journal 'total:staked' key");
			return 0;
		}
	};

	let last_total_liquid_key = match last_total_key.get("liquid") {
		Some(value) => value,
		None => {
			println!("Missing or invalid last journal 'total:liquid' key");
			return 0;
		}
	};

	let last_total_liquid = match last_total_liquid_key.get("NOM")
		.and_then(Value::as_str)
		.and_then(|s| s.parse::<u64>().ok()) {
		Some(value) => value,
		None => {
			println!("Missing or invalid last journal 'total:liquid:NOM' key");
			return 0;
		}
	};

	calculate_daily_reward(
		timestamp,
		total_staked,
		total_liquid,
		last_timestamp,
		last_total_staked,
		last_total_liquid,
	)
}

fn calculate_stake(
    liquid: u64,
    balance: u64,
    claim_fee: u64,
    stake_fee: u64,
    minimum_balance: u64,
    staked: u64,
    minimum_stake: u64,
    adjust: &str,
    daily_reward: u64,
    rounding: f64,
) -> (String, u64) {
    
    // Define the closure
    let calculate_quantity = |available: u64, alignment: u64, quantum: u64| -> u64 {
        if available > alignment {
            (((available - alignment) / quantum) * quantum) + alignment
        } else {
            0
        }
    };

    // Define stake_quantum
    let stake_quantum = if adjust == "true" && daily_reward > 0 {
        let rounding_unom = (rounding * 1_000_000.0) as u64;
        (daily_reward / rounding_unom) * rounding_unom
    } else {
        minimum_stake
    };

    // Calculate quantities
    let quantity_without_claim = calculate_quantity(
        balance - minimum_balance - stake_fee,
        staked % stake_quantum,
        stake_quantum,
    );

    let quantity_after_claim = calculate_quantity(
        liquid + balance - minimum_balance - claim_fee - stake_fee,
        staked % stake_quantum,
        stake_quantum,
    );

    // Return results based on the computed quantities
    if quantity_without_claim > 0 {
        return ("false".to_string(), quantity_without_claim);
    }

    if quantity_after_claim > 0 {
        return ("true".to_string(), quantity_after_claim);
    }
    
    ("false".to_string(), 0)
}

pub fn stake(journal: &Map<String, Value>) -> Value  {

	let minimum_balance = minimum_balance(&journal);
	let daily_reward = daily_reward(&journal);
    let null_data: Value = json!({
        "minimum_balance": minimum_balance,
        "daily_reward": daily_reward,
        "claim": "false",
        "stake": 0
    });

	let balance_key = match journal.get("balance") {
		Some(value) => value,
		None => {
			println!("Missing or invalid 'balance' key");
			return null_data;
		}
	};

	let balance = match balance_key.get("NOM")
		.and_then(Value::as_str)
		.and_then(|s| s.parse::<u64>().ok()) {
		Some(value) => value,
		None => {
			println!("Missing or invalid 'balance:NOM' key");
			return null_data;
		}
	};

	let total_key = match journal.get("total") {
		Some(value) => value,
		None => {
			println!("Missing or invalid 'total' key");
			return null_data;
		}
	};

	let liquid_key = match total_key.get("liquid") {
		Some(value) => value,
		None => {
			println!("Missing or invalid 'total:liquid' key");
			return null_data;
		}
	};

	let total_liquid = match liquid_key.get("NOM")
		.and_then(Value::as_str)
		.and_then(|s| s.parse::<u64>().ok()) {
		Some(value) => value,
		None => {
			println!("Missing or invalid 'total:liquid:NOM' key");
			return null_data;
		}
	};

	let config_key = match journal.get("config") {
		Some(value) => value,
		None => {
			println!("Missing or invalid 'config' key");
			return null_data;
		}
	};

    let config_minimum_stake = match config_key.get("MINIMUM_STAKE")
        .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())) {
        Some(value) => (value * 1_000_000.0) as u64,
        None => {
            println!("missing or invalid 'config:MINIMUM_STAKE' key");
            println!("Using global default: {}", *MINIMUM_STAKE);
			(*MINIMUM_STAKE * 1_000_000.0) as u64
        }
    };

    let rounding = match config_key.get("MINIMUM_STAKE_ROUNDING")
        .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())) {
        Some(value) => value,
        None => {
            println!("missing or invalid 'config:MINIMUM_STAKE_ROUNDING' key");
            println!("Using global default: {}", *MINIMUM_STAKE_ROUNDING);
			*MINIMUM_STAKE_ROUNDING
        }
    };

    let adjust = match config_key.get("ADJUST_MINIMUM_STAKE") {
		Some(value) => value.as_str().unwrap_or(""),
		None => {
			println!("Missing 'config:ADJUST_MINIMUM_STAKE' key");
            println!("Using global default: {}", *ADJUST_MINIMUM_STAKE);
			&*ADJUST_MINIMUM_STAKE
		}
    };

	let validator = match config_key.get("VALIDATOR") {
		Some(value) => value.as_str().unwrap_or(""),
		None => {
			println!("Missing 'config:VALIDATOR' key");
			return null_data;
		}
    };

	let delegations_key = match journal.get("delegations") {
		Some(value) => value,
		None => {
			println!("Missing or invalid 'delegations' key");
			return null_data;
		}
	};

	let validator_key = match delegations_key.get(validator) {
		Some(value) => value,
		None => {
			println!("Missing or invalid 'delegations:{}' key", validator);
			return null_data;
		}
	};

	let staked = match validator_key.get("staked")
		.and_then(Value::as_str)
		.and_then(|s| s.parse::<u64>().ok()) {
		Some(value) => value,
		None => {
			println!("Missing or invalid 'delegations:{}:staked' key", validator);
			return null_data;
		}
	};

	let (claim, stake) = calculate_stake(
		total_liquid,
		balance,
		(*CLAIM_FEE * 1_000_000.0) as u64,
		(*STAKE_FEE * 1_000_000.0) as u64,
		minimum_balance,
		staked,
		config_minimum_stake,
		&adjust,
		daily_reward,
		rounding,
	);

    json!({
        "minimum_balance": minimum_balance,
        "daily_reward": daily_reward,
        "claim": claim,
        "stake": stake
    })
}
