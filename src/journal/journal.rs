use chrono::{DateTime, Utc};
use clap::ValueEnum;
use colored::Colorize;
use crate::functions::format_to_millions;
use crate::globals::GlobalConfig;
use eyre::{Result, WrapErr};
use indexmap::IndexMap;
//use log::warn;
use num_format::ToFormattedString;
use serde_json::{Value, to_value};
use std::str::FromStr;

/// A wrapper around `Value` that implements custom display formatting.
#[derive(Debug)]
struct DisplayLogValue(Value);

#[allow(dead_code)]
#[derive(Debug)]
struct DisplayLogOptionValue(Option<Value>);

impl DisplayLogValue {
    /// Common display formatting logic for `DisplayLogValue`.
    fn format_value(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Value::Number(n) => {
                // Check for u64
                if let Some(num) = n.as_u64() {
                    let value = num as f64 / 1_000_000.0; // Convert to millions
                    if value < 100.0 {
                        write!(f, "{:.2}", value) // Format as 2 decimal places
                    } else {
                        let integer_part = value.trunc() as u64;
                        write!(f, "{}", integer_part.to_formatted_string(&num_format::Locale::en))
                    }
                } else {
                    write!(f, "{}", n) // Return the unchanged input for other cases
                }
            }
            Value::String(s) => {
                // Try to parse the string as a datetime
                if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                    let dt_utc = dt.with_timezone(&Utc);
                    write!(f, "{}", dt_utc.format("%m-%d %H:%M")) // Format the date
                } else {
                    write!(f, "{}", s) // Return the original string if invalid
                }
            }
            _ => write!(f, "{}", self.0), // Return the original value for non-string cases
        }
    }
}

/// Implements the `Display` trait for `DisplayLogValue`.
/// This allows for custom formatting when the struct is printed.
impl std::fmt::Display for DisplayLogValue {
    /// Formats the contained `Value` based on its type.
    ///
    /// - For `Value::Number`, it attempts to interpret the number as a `u64`
    ///   and divides it by 1,000,000, displaying the result in millions.
    ///   If the result is less than 100, it is displayed with two decimal places.
    ///   Otherwise, it is formatted as an integer with proper thousand separators.
    ///
    /// - For `Value::String`, it tries to parse the string as an RFC 3339 datetime.
    ///   If parsing succeeds, it formats the datetime as "MM-DD HH:MM" in UTC.
    ///   If parsing fails, the original string is displayed.
    ///
    /// - For other value types, it defaults to using the underlying value's
    ///   `Display` implementation.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format_value(f)
    }
}

impl std::fmt::Display for DisplayLogOptionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = &self.0 {
            DisplayLogValue(value.clone()).fmt(f) // Use the helper function for formatting
        } else {
            write!(f, "None") // Handle the case for None
        }
    }
}

#[derive(Debug)]
struct DisplayJournalValue(Value);

impl std::fmt::Display for DisplayJournalValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Value::String(s) => {
                // Try to parse the string as a datetime
                if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                    let dt_utc = dt.with_timezone(&Utc);
                    write!(f, "{}", dt_utc.format("%m-%d %H:%M"))
                } else {
                    write!(f, "{}", s) // Return the original string if invalid
                }
            },
            Value::Number(n) => {
                // Check for u64
                if let Some(num) = n.as_u64() {
                    write!(f, "{}", format_to_millions(num, None))
                } else {
                    write!(f, "{}", n) // Return the unchanged input for other cases
                }
            },
            _ => write!(f, "{}", self.0), // Return the original value for non-string cases
        }
    }
}

struct DateDisplayFromOption(Option<DateTime<Utc>>);

impl std::fmt::Display for DateDisplayFromOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(dt) => {
                // Format the DateTime as "MM-DD_HH:MM"
                let formatted = dt.format("%m-%d %H:%M").to_string();
                write!(f, "{}", formatted)
            }
            None => write!(f, "N/A"), // Default to "N/A" if None
        }
    }
}

/// Enum to represent output formats
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    JsonPretty,
    List,
    Log,
}

impl FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json"        => Ok(OutputFormat::Json),
            "json-pretty" => Ok(OutputFormat::JsonPretty),
            "list"        => Ok(OutputFormat::List),
            "log"         => Ok(OutputFormat::Log),
            _             => Err(format!("Invalid output format: {}", s)),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            OutputFormat::Json       => "json",
            OutputFormat::JsonPretty => "json-pretty",
            OutputFormat::List       => "list",
            OutputFormat::Log        => "log",
        };
        write!(f, "{}", output)
    }
}

// Define the FromValue trait
pub trait FromValue: Sized {
    fn from_value(value: &Value) -> Option<Self>;
}

// Implement FromValue for DateTime<Utc>
impl FromValue for DateTime<Utc> {
    fn from_value(value: &Value) -> Option<Self> {
        value.as_str()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc)) // Convert to UTC
    }
}

// Implementations for different types
impl FromValue for String {
    fn from_value(value: &Value) -> Option<Self> {
        value.as_str().map(|s| s.to_string()) // Convert to String
    }
}

impl FromValue for u64 {
    fn from_value(value: &Value) -> Option<Self> {
        value.as_u64() // Convert to u64 if possible
    }
}

impl FromValue for f64 {
    fn from_value(value: &Value) -> Option<Self> {
        value.as_f64() // Convert to f64 if possible
    }
}

// Define the Journal struct with an IndexMap
#[derive(Clone)]
pub struct Journal(IndexMap<String, Value>);

// Implement the Display trait for Journal
impl std::fmt::Display for Journal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in &self.0 {
            write!(f, "{:width$} : ", key, width = self.max_key_length())?; // Print key with padding
            if key == "rank" {
                writeln!(f, "{}", value.clone())?;
            } else {
                writeln!(f, "{}", DisplayJournalValue(value.clone()))?;
            }
        }

        Ok(())
    }
}

// Implement the Debug trait for Journal
impl std::fmt::Debug for Journal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in &self.0 {
            write!(f, "{}: ", key)?; // Print key without padding
            match value {
                Value::String(s) => writeln!(f, "{}", s)?,
                Value::Number(n) => writeln!(f, "{}", n)?,
                Value::Bool(b) => writeln!(f, "{}", b)?,
                Value::Array(arr) => writeln!(f, "{:?}", arr)?,
                Value::Object(obj) => writeln!(f, "{:#?}", obj)?,
                Value::Null => writeln!(f, "null")?,
            }
        }
        Ok(())
    }
}

// Implement the Journal struct
impl Journal {

    // Method to create an empty Journal
    pub fn new() -> Self {
        Self(IndexMap::new())
    }

    // Method to create a Journal instance from a JSON string
    pub fn from_json_str(json_str: &str) -> Result<Self> {
        let trimmed_str = json_str.trim(); // Trim whitespace
        let index_map: IndexMap<String, Value> = serde_json::from_str(trimmed_str)
            .wrap_err("Failed to parse output as JSON")?;
        Ok(Journal(index_map))
    }

    // Method to add an entry to the Journal
    pub fn insert(&mut self, key: String, value: Value) {
        self.0.insert(key, value);
    }

    // Method to retrieve values based on the FromValue trait
    pub fn get<T: FromValue>(&self, key: &str) -> Option<T> {
        self.0.get(key).and_then(|value| T::from_value(value))
    }

    // Method to serialize the IndexMap to JSON
    pub fn json(&self) -> Result<Value> {
        to_value(&self.0).wrap_err("Failed to serialize IndexMap to JSON")
    }

    // Max width of all keys
    fn max_key_length(&self) -> usize {
        // Find the maximum length of keys in the collection
        self.0.keys().map(|key| key.len()).max().unwrap_or(0)
    }

    pub fn log(&self) -> String {

        // Attempt to load config or use the default if loading fails
        let config = match GlobalConfig::load() {
            Ok(config) => config,
            Err(err) => {
                eprintln!("Error loading config: {}. Using default column widths.", err);
                let mut default_config = GlobalConfig::new();
                default_config.log.column_widths = vec![11, 1, 8, 7, 7, 6, 6, 7, 8, 8, 9, 7];
                let _ = default_config.save(); // Ignore errors for simplicity
                default_config // Use the default config
            }
        };

        let col = &config.log.column_widths; // Retrieve the column widths

        let mut output = String::new();

        // Extract values
        let staked = self.get::<String>("staked").unwrap_or("❌".to_string());
        let is_staked = staked == "✅";
        let quantity = self.get::<u64>("quantity_to_stake").unwrap_or(0);
        let total_liquid = self.get::<u64>("total_liquid").unwrap_or(0);
        let remaining = self.get::<u64>("remaining").unwrap_or(0);
        let minimum_stake = self.get::<u64>("minimum_stake").unwrap_or(0);
        let balance = self.get::<u64>("balance").unwrap_or(0);

        // Calculations
//        let remaining_to_stake = balance
//            .saturating_add(minimum_stake)
//            .saturating_sub(validator_staked_remainder)
//            .saturating_sub(total_liquid);
        let minimum_stake_value = if is_staked { minimum_stake } else { remaining };
//        let minimum_stake_value = minimum_stake;
        let total_liquid_value = if is_staked { quantity } else { total_liquid };

        // Formatting and color handling for each column
        output = format!(
            "{}{}│", 
            output, 
            pad_or_truncate(&format!("{}", DateDisplayFromOption(self.get("timestamp"))), col[0], false)
        );

        output = format!(
            "{}{}│", 
            output, 
            pad_or_truncate(&staked, col[1], false)
        );

        output = format!(
            "{}{}", 
            output, 
            pad_or_truncate(&self.get::<String>("profile").unwrap_or("N/A".to_string()), col[2], false)
        );

        // Apply color based on individual conditions for each cell
        let total_staked_str = pad_or_truncate(
            &format!("{}", DisplayLogValue(self.get::<u64>("total_staked").unwrap_or(0).into())), 
            col[3], 
            true
        ).green();
        output = format!("{}{}", output, total_staked_str);

        let daily_reward_str = pad_or_truncate(
            &format!("{}", DisplayLogValue(self.get::<u64>("daily_reward").into())),
            col[4],
            true
        ).blue();
        output = format!("{}{}", output, daily_reward_str);

        let minimum_balance_str = pad_or_truncate(
            &format!("{}", DisplayLogValue(self.get::<u64>("minimum_balance").into())),
            col[5],
            true
        ).magenta();
        output = format!("{}{}", output, minimum_balance_str);

        let balance_str = pad_or_truncate(
            &format!("{}", DisplayLogValue(balance.into())), 
            col[6], 
            true
        ).green();
        output = format!("{}{}", output, balance_str);

        let total_liquid_str = pad_or_truncate(
            &format!("{}", DisplayLogValue(total_liquid_value.into())), 
            col[7], 
            true
        );
        let total_liquid_str_colored = if is_staked {
            total_liquid_str.green()
        } else {
            total_liquid_str.yellow()
        };
        output = format!("{}{}", output, total_liquid_str_colored);

        let minimum_stake_str = pad_or_truncate(
            &format!("{}", DisplayLogValue(minimum_stake_value.into())),
            col[8],
            true
        );
        let minimum_stake_str_colored = if is_staked {
            minimum_stake_str.blue()
        } else {
            minimum_stake_str.truecolor(255, 165, 0)  // Orange color for unstaked
        };
        output = format!("{}{}│", output, minimum_stake_str_colored);

        let config_validator_name_str = pad_or_truncate(
            &format!("{}", DisplayLogValue(self.get::<String>("config_validator_name").into())),
            col[9],
            false
        );
        output = format!("{}{}", output, config_validator_name_str);

        let voting_power_str = pad_or_truncate(
            &format!("{}", DisplayLogValue(self.get::<u64>("voting_power").into())),
            col[10],
            true
        ).magenta();
        output = format!("{}{}", output, voting_power_str);

        let validator_staked_str = pad_or_truncate(
            &format!("{}", DisplayLogValue(self.get::<u64>("validator_staked").unwrap_or(0).into())),
            col[11],
            true
        ).green();
        output = format!("{}{}", output, validator_staked_str);

        output
    }

    pub fn print(&self,
        format: Option<OutputFormat>,
    ) -> eyre::Result<()> {

        // Use the default format if None is provided
        let format = format.unwrap_or(OutputFormat::List);

        match format {
            OutputFormat::Json => {
                let json_value = self.json()?;
                let json_str = serde_json::to_string(&json_value)
                    .map_err(|e| eyre::eyre!("Error serializing JSON: {}", e))?;
                println!("{}", json_str);
            },
            OutputFormat::JsonPretty => {
                let json_value = self.json()?;
                let pretty_json = serde_json::to_string_pretty(&json_value)
                    .map_err(|e| eyre::eyre!("Error serializing JSON: {}", e))?;
                println!("{}", pretty_json);
            },
            OutputFormat::List => {
                println!("{}", self);
            },
            OutputFormat::Log => {
                println!("{}", self.log());
            },
        }
        Ok(())
    }
}

fn pad_or_truncate(s: &str, width: usize, right_align: bool) -> String {
    let len_without_ansi = s.chars().filter(|&c| !c.is_ascii_control()).count(); // Ignore ANSI escape codes

    if len_without_ansi > width {
        // Truncate if the string is too long
        s.chars().take(width).collect()
    } else {
        // Add padding
        let padding = " ".repeat(width - len_without_ansi);
        if right_align {
            format!("{}{}", padding, s) // Right-align
        } else {
            format!("{}{}", s, padding) // Left-align
        }
    }
}
