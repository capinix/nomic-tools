use eyre::{Result, WrapErr};
use serde_json::Value;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use chrono::DateTime;
use fmt::text::text;
use colored::*;


// Helper function to format a value with custom width and decimal places
fn format_value(value: f64, width: usize) -> String {
    let decimal_places = if value < 100.0 {
        2 
    } else {
        0
    };
    text(
        Some(&value.to_string()),      // Pass the raw value as a string
        Some(width),                   // Specify width
        None,                          // No wrap or truncate
        None,                          // No ellipsis
        Some(true),                    // Pad decimal digits
        Some(decimal_places),          // Max decimal digits
        None,                          // No specific decimal separator
        Some(true),                    // Use thousand separator
        None,                          // No specific thousand separator
        None                           // Default alignment
    )
}

// Define the ApplyIf trait
pub trait ApplyIf<T>: Sized {
    fn apply_if(self, condition: bool, f: impl FnOnce(Self) -> T) -> T;
}

// Implement ApplyIf for String
impl ApplyIf<String> for String {
    fn apply_if(self, condition: bool, f: impl FnOnce(Self) -> String) -> String {
        if condition {
            f(self)
        } else {
            self
        }
    }
}

fn read_line(line: &str) {

    // Attempt to parse the line as JSON
    let json: Value = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Failed to parse line as JSON: {}", line);
            return; // Skip this line and continue to the next one
        }
    };

    // Extract and handle relevant data
    let timestamp = json.get("timestamp").and_then(Value::as_str).unwrap_or("N/A");
    let staked = json.get("staked").and_then(Value::as_str).unwrap_or("N/A");
    let profile = json.get("profile").and_then(Value::as_str).unwrap_or("N/A");

    // Extract numerical values with default fallbacks
    let total_staked = json.get("total_staked").and_then(Value::as_u64).unwrap_or(0);
    let daily_reward = json.get("daily_reward").and_then(Value::as_f64).unwrap_or(0.0);
    let minimum_balance = json.get("minimum_balance").and_then(Value::as_u64).unwrap_or(0);
    let minimum_stake = json.get("minimum_stake").and_then(Value::as_u64).unwrap_or(0);
    let balance = json.get("balance").and_then(Value::as_u64).unwrap_or(0);
    let total_liquid = json.get("total_liquid").and_then(Value::as_u64).unwrap_or(0);
    let quantity_to_stake = json.get("quantity_to_stake").and_then(Value::as_u64).unwrap_or(0);
    let config_validator_moniker = json.get("config_validator_moniker").and_then(Value::as_str).unwrap_or("N/A");
    let voting_power = json.get("voting_power").and_then(Value::as_u64).unwrap_or(0);
    let validator_staked = json.get("validator_staked").and_then(Value::as_u64).unwrap_or(0);
    let validator_staked_remainder = json.get("validator_staked_remainder").and_then(Value::as_u64).unwrap_or(0);
    // let claim_fee = json.get("claim_fee").and_then(Value::as_u64).unwrap_or(0);
    // let stake_fee = json.get("stake_fee").and_then(Value::as_u64).unwrap_or(0);
    let available_after_claim = json.get("available_after_claim").and_then(Value::as_u64).unwrap_or(0);

    // Format the timestamp
    let formatted_date = DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.format("%m-%d %H:%M").to_string())
        .unwrap_or_else(|_| "Invalid Date".to_string());

    let base = if validator_staked_remainder > 0 {
        validator_staked_remainder
    } else {
        minimum_stake
    };

    let remaining_required = base.saturating_sub(available_after_claim);

    // Custom width and decimal places for each column
    let formatted_total_staked    = format_value(total_staked      as f64 / 1_000_000.0, 8);
    let formatted_daily_reward    = format_value(daily_reward      as f64 / 1_000_000.0, 7);
    let formatted_minimum_balance = format_value(minimum_balance   as f64 / 1_000_000.0, 7);
    let formatted_balance         = format_value(balance           as f64 / 1_000_000.0, 7);
    let formatted_liquid_or_stake = if staked == "✅" {
        format_value(quantity_to_stake as f64 / 1_000_000.0, 7)
    } else {
        format_value(total_liquid      as f64 / 1_000_000.0, 7)
    }
    .apply_if(staked == "✅", |text| text.green().to_string())
    .apply_if(staked != "✅", |text| text.yellow().to_string());
    let formatted_left_or_stake = if staked == "✅" {
        format_value(minimum_stake as f64 / 1_000_000.0, 7)
    } else {
        format_value(remaining_required as f64 / 1_000_000.0, 7)
    }
    .apply_if(staked == "✅", |text| text.green().to_string())
    .apply_if(staked != "✅", |text| text.truecolor(255, 165, 0).to_string());
    //..apply_if(staked != "✅", |text| text.bright_green().to_string());
    let formatted_voting_power = format_value(voting_power as f64 / 1_000_000.0, 8);
    let formatted_validator_staked = format_value(validator_staked as f64 / 1_000_000.0, 7);

    // Print the formatted output
    println!("{:10}│{:1}│{:8}{:7}{:7}{:7}{:7}{:7}{:7}│{:8}{:8}{:8}",
        formatted_date,
        staked,
        profile,
        formatted_total_staked.green(),
        formatted_daily_reward.blue(),
        formatted_minimum_balance.purple(),
        formatted_balance.cyan(),
        formatted_liquid_or_stake,
        formatted_left_or_stake,
        config_validator_moniker,
        formatted_voting_power.magenta(),
        formatted_validator_staked.green(),
    );
}


pub fn tail_journalctl(staked_or_not: Option<bool>) -> Result<()> {
    // Get the path of the current executable
    let exe_path = env::current_exe().wrap_err("Failed to get the current executable path")?;
    let exe_path_str = exe_path.to_string_lossy();

    // Prepare the grep expression to match JSON objects containing the "address" key
    // let grep_expr = r#"{[^}]*"address"[[:space:]]*:[[:space:]]*"[^"]*"[^}]*}"#;
    // Prepare the grep expression to match based on staked status
    let grep_expr = match staked_or_not {
        Some(true) => r#"{[^}]*"staked"[[:space:]]*:[[:space:]]*"✅"[^}]*}"#,
        Some(false) => r#"{[^}]*"staked"[[:space:]]*:[[:space:]]*"❌"[^}]*}"#,
        None => r#"{[^}]*"staked"[[:space:]]*:[[:space:]]*"(✅|❌)"[^}]*}"#,
    };


    // Start the journalctl command
    let mut child = Command::new("journalctl")
        .args(&[
            &format!("_EXE={}", exe_path_str),
            &format!("--grep={}", grep_expr),
            "--output=cat",
            "--no-pager",
            "--follow",
            "--lines=20",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .wrap_err("Failed to start journalctl")?;

    // Read stdout from journalctl
    let reader = BufReader::new(child.stdout.take().unwrap());

    for line in reader.lines() {
        let line = line.wrap_err("Failed to read line from journalctl")?;

        // Pass the raw line to read_line
        read_line(&line);
    }

    // Wait for the child process to finish
    let status = child.wait().wrap_err("Failed to wait for journalctl process")?;
    if !status.success() {
        return Err(eyre::eyre!("journalctl process exited with a non-zero status"));
    }

    Ok(())
}
