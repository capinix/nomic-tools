use eyre::{Result, WrapErr};
use serde_json::Value;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use chrono::DateTime;
use fmt::text::text;
use colored::*;

pub fn tail_journalctl() -> Result<()> {
    // Get the path of the current executable
    let exe_path = env::current_exe()
        .wrap_err("Failed to get the current executable path")?;

    // Convert the path to a string
    let exe_path_str = exe_path.to_string_lossy();

    // Prepare the grep expression to match entire JSON objects containing the "address" key
    let grep_expr = r#"{[^}]*"address"[[:space:]]*:[[:space:]]*"[^"]*"[^}]*}"#;

    // Command to tail journalctl output
    let mut child = Command::new("journalctl")
        .args(&[
            &format!("_EXE={}", exe_path_str),
            &format!("--grep={}", grep_expr),
            "--output=cat", // Output as plain text
            "--no-pager",
            "--follow",
            "--lines=26",
        ])
        .stdout(Stdio::piped()) // Pipe stdout so we can read it
        .spawn()
        .wrap_err("Failed to start journalctl")?; // Wrap errors for better diagnostics

    // Read the stdout from journalctl
    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.wrap_err("Failed to read line from journalctl")?;

        // Parse the line as JSON
        let json: Value = serde_json::from_str(&line)
            .wrap_err("Failed to parse output as JSON")?;

        // Extract relevant data
        let timestamp = json.get("timestamp").and_then(|v| v.as_str()).unwrap_or("N/A");
        let staked = json.get("staked").and_then(|v| v.as_str()).unwrap_or("N/A");
        let profile = json.get("profile").and_then(|v| v.as_str()).unwrap_or("N/A");
        let total_staked = json.get("total_staked").and_then(|v| v.as_u64()).unwrap_or(0);
        let daily_reward = json.get("daily_reward").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let minimum_balance = json.get("minimum_balance").and_then(|v| v.as_u64()).unwrap_or(0);
        let balance = json.get("balance").and_then(|v| v.as_u64()).unwrap_or(0);
        let total_liquid = json.get("total_liquid").and_then(|v| v.as_u64()).unwrap_or(0);
        let quantity_to_stake = json.get("quantity_to_stake").and_then(|v| v.as_u64()).unwrap_or(0);
        let config_minimum_stake = json.get("config_minimum_stake").and_then(|v| v.as_u64()).unwrap_or(0);
        let config_validator_moniker = json.get("config_validator_moniker").and_then(|v| v.as_str()).unwrap_or("N/A");
        let voting_power = json.get("voting_power").and_then(|v| v.as_u64()).unwrap_or(0);
        let validator_staked = json.get("validator_staked").and_then(|v| v.as_u64()).unwrap_or(0);


        // Parse and format the timestamp, using a default if parsing fails
        let formatted_date = match DateTime::parse_from_rfc3339(timestamp) {
            Ok(parsed) => parsed.format("%m-%d %H:%M").to_string(),
            Err(_) => "Invalid Date".to_string(), // Default value for bad timestamps
        };

        // Convert total_staked from string to f64 for calculation
        // let total_staked_value: f64 = total_staked.parse().unwrap_or(0.0);

        // Format total staked value divided by 1,000,000
        // let formatted_staked = format!("{:.0}", (total_staked / 1_000_000));
        let formatted_staked = text(
            Some(&(total_staked as f64 / 1_000_000.0).to_string()),   // input string
            Some(8),                                        // width
            None,                                            // wrap or truncate,
            None,                                            // no ellipsis
            Some(true),                                      // pad_decimal_digits
            Some(0),                                         // max_decimal_digits
            None,                                            // decimal_separator
            Some(true),                                      // use_thousand_separator
            None,                                            // thousand_separator
            None,                                            // alignment
            //Some(fmt::text::Alignment::RIGHT),               // alignment
        );
        let formatted_reward = text(
            Some(&(daily_reward / 1_000_000.0).to_string()),
            Some(8),
            None, None, Some(true),
            Some(2),
            None, Some(true), None, None,
        );
        let formatted_minimum_balance = text(
            Some(&((minimum_balance as f64 / 1_000_000.0) as f64).to_string()),
            Some(8),
            None, None, Some(true),
            Some(2),
            None, Some(true), None, None,
        );

        let formatted_balance = text(
            Some(&((balance as f64 / 1_000_000.0) as f64).to_string()),
            Some(8),
            None, None, Some(true),
            Some(2),
            None, Some(true), None, None,
        );

        let formatted_liquid_or_stake = if staked == "✅" {
            text(
                Some(&((quantity_to_stake as f64 / 1_000_000.0) as f64).to_string()),
                Some(8),
                None, None, Some(true),
                Some(2),
                None, Some(true), None, None,
            )
        } else {
            text(
                Some(&((total_liquid as f64 / 1_000_000.0) as f64).to_string()),
                Some(8),
                None, None, Some(true),
                Some(2),
                None, Some(true), None, None,
            )
        };
        let formatted_config_minimum_stake = text(
            Some(&((config_minimum_stake as f64 / 1_000_000.0) as f64).to_string()),
            Some(8),
            None, None, Some(true),
            Some(2),
            None, Some(true), None, None,
        );

        let formatted_voting_power = text(
            Some(&((voting_power as f64 / 1_000_000.0) as f64).to_string()),
            Some(8),
            None, None, Some(true),
            Some(0),
            None, Some(true), None, None,
        );

        let formatted_validator_staked = text(
            Some(&((validator_staked as f64 / 1_000_000.0) as f64).to_string()),
            Some(8),
            None, None, Some(true),
            Some(0),
            None, Some(true), None, None,
        );
        let formatted_liquid_or_stake = if staked == "✅" { 
            formatted_liquid_or_stake.green() 
        } else { 
            formatted_liquid_or_stake.yellow() 
        };


        // Print the formatted output
        println!("{:10}│{:1}│{:8}{:8}{:8}{:8}{:8}{:8}{:8}│{:8}{:8}{:8}", 
            formatted_date,
            staked,
            profile,
            formatted_staked.green(),
            formatted_reward.blue(),
            formatted_minimum_balance.purple(),
            formatted_balance.green(),
            formatted_liquid_or_stake,
            formatted_config_minimum_stake,
            config_validator_moniker,
            formatted_voting_power.magenta(),
            formatted_validator_staked.cyan(),
        );

    }

    // Wait for the child process to finish
    let status = child.wait().wrap_err("Failed to wait for journalctl process")?;
    if !status.success() {
        return Err(eyre::eyre!("journalctl process exited with a non-zero status"));
    }

    Ok(())
}

