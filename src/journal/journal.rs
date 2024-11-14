use chrono::{DateTime, Local, Utc};
use clap::ValueEnum;
use colored::Colorize;
use crate::functions::NumberDisplay;
use crate::functions::pad_or_truncate;
use crate::global::CONFIG;
use eyre::{Result, WrapErr};
use indexmap::IndexMap;
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
                    let dt_local = dt.with_timezone(&Local);
                    write!(f, "{}", dt_local.format("%m-%d %H:%M")) // Format the date
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
                    let dt_local = dt.with_timezone(&Local);
                    write!(f, "{}", dt_local.format("%m-%d %H:%M"))
                } else {
                    write!(f, "{}", s) // Return the original string if invalid
                }
            },
            Value::Number(n) => {
                // Check for u64
                if let Some(num) = n.as_u64() {
                    write!(f, "{}", NumberDisplay::new(num).scale(6).decimal_places(6).trim(true).format())
                } else {
                    write!(f, "{}", n) // Return the unchanged input for other cases
                }
            },
            _ => write!(f, "{}", self.0), // Return the original value for non-string cases
        }
    }
}

//struct DateDisplayFromOption(Option<DateTime<Utc>>);
//
//impl std::fmt::Display for DateDisplayFromOption {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        match &self.0 {
//            Some(dt) => {
//                // Convert to local timezone and format
//                let dt_local = dt.with_timezone(&Local);
//                write!(f, "{}", dt_local.format("%m-%d %H:%M"))
//            }
//            None => write!(f, "N/A"), // Default to "N/A" if None
//        }
//    }
//}

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

        let col = CONFIG.journalctl.tail.column_widths.clone();

        // Extract values
        let timestamp = self.get::<String>("timestamp")
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok()
                .map(|dt| dt.with_timezone(&Local).format("%m-%d %H:%M").to_string())
                .or(Some(s)))
            .unwrap_or_else(|| "N/A".to_string());
        let staked                  = self.get::<String>("staked").unwrap_or("❌".to_string());
        let claimed                 = self.get::<String>("claimed").unwrap_or("❌".to_string());
        let balance                 = self.get::<u64>("balance").unwrap_or(0);
        let total_staked            = self.get::<u64>("total_staked").unwrap_or(0);
        let total_liquid            = self.get::<u64>("total_liquid").unwrap_or(0);
        let validator_staked        = self.get::<u64>("validator_staked").unwrap_or(0);
        let validator_name          = self.get::<String>("config_validator_name").unwrap_or("N/A".to_string());
        let voting_power            = self.get::<u64>("voting_power").unwrap_or(0);
        let minimum_balance         = self.get::<u64>("minimum_balance").unwrap_or(0);
        let minimum_stake           = self.get::<u64>("minimum_stake").unwrap_or(0);
        let daily_reward            = self.get::<u64>("daily_reward").unwrap_or(0);
        let available_without_claim = self.get::<u64>("available_without_claim").unwrap_or(0);
        let available_after_claim   = self.get::<u64>("available_after_claim").unwrap_or(0);
        let quantity                = self.get::<u64>("quantity").unwrap_or(0);
        let remaining               = self.get::<u64>("remaining").unwrap_or(0);
        let profile                 = self.get::<String>("profile").unwrap_or("N/A".to_string());

        let is_staked  = staked  == "✅";
        let is_claimed = claimed == "✅";

        // Date
        let col0 = pad_or_truncate(timestamp, col[0], false);

        // action
        let col1 = pad_or_truncate(staked, col[1], false);

        // profile
        let col2 = pad_or_truncate(&profile, col[2], false);

        // total staked
        let col3 = if is_staked {
            pad_or_truncate(&NumberDisplay::new(total_staked + quantity)
                .scale(6).decimal_places(2).integer_threshold(100).trim(true).format(),
                col[3], true
            ).yellow()
        } else {
            pad_or_truncate(&NumberDisplay::new(total_staked)
                .scale(6).decimal_places(2).integer_threshold(100).trim(true).format(),
                col[3], true
            ).green()
        };

        // daily reward
        let col4 = pad_or_truncate(&NumberDisplay::new(daily_reward)
            .scale(6).decimal_places(2).integer_threshold(100).trim(true).format(),
            col[4], true
        ).magenta();

        // minimum_stake
        let col5 = pad_or_truncate(&NumberDisplay::new(minimum_stake)
            .scale(6).decimal_places(2).integer_threshold(100).trim(true).format(),
            col[5], true
        ).blue();

        // minimum_balance
        let col6 = pad_or_truncate(&NumberDisplay::new(minimum_balance)
            .scale(6).decimal_places(2).integer_threshold(100).format(),
            col[6], true
        ).blue();

        // balance
        let col7 = if is_staked {
            let adjusted_balance = if is_claimed {
                minimum_balance.saturating_add(available_after_claim).saturating_sub(quantity)

            } else {
                minimum_balance.saturating_add(available_without_claim).saturating_sub(quantity)
            };
            pad_or_truncate(&NumberDisplay::new(adjusted_balance)
                .scale(6).decimal_places(2).integer_threshold(100).format(),
                col[7], true
            ).yellow()
        } else {
            pad_or_truncate(&NumberDisplay::new(balance)
                .scale(6).decimal_places(2).integer_threshold(100).format(),
                col[7], true
            ).green()
        };

        // total_liquid
        let col8 = if is_staked {
            pad_or_truncate("", col[8], false).green()
        } else {
            pad_or_truncate(&NumberDisplay::new(total_liquid)
                .scale(6).decimal_places(2).integer_threshold(100).format(),
                col[8], true
            ).green()
        };

        // staked quantity / remaining to stake
        let col9 = if is_staked {
            // staked quantity
            pad_or_truncate(&NumberDisplay::new(quantity)
                .scale(6).decimal_places(2).integer_threshold(100).trim(true).format(),
                col[9], true
            ).yellow()
        } else {
            // remining to stake
            pad_or_truncate(&NumberDisplay::new(remaining)
                .scale(6).decimal_places(2).integer_threshold(100).format(),
                col[9], true
            ).truecolor(255, 165, 0)
        };

        let col10 = pad_or_truncate(&validator_name, col[10], false);

        let col11 = if is_staked {
            pad_or_truncate(&NumberDisplay::new(voting_power + quantity)
                .scale(6).decimal_places(2).integer_threshold(100).trim(true).format(),
                col[11], true
            ).yellow()
        } else {
            pad_or_truncate(&NumberDisplay::new(voting_power)
                .scale(6).decimal_places(2).integer_threshold(100).trim(true).format(),
                col[11], true
            ).magenta()
        };

        let col12 = if is_staked {
            pad_or_truncate(&NumberDisplay::new(validator_staked + quantity)
                .scale(6).decimal_places(2).integer_threshold(100).trim(true).format(),
                col[12], true
            ).yellow()
        }  else {
            pad_or_truncate(&NumberDisplay::new(validator_staked)
                .scale(6).decimal_places(2).integer_threshold(100).trim(true).format(),
                col[12], true
            ).green()
        };

        format!(
            "{}│{}│{}{}│{}{}│{}{}│{}{}│{}{}{}",
            col0, col1, col2, col3, col4, col5, col6, col7, col8, col9, col10, col11, col12
        )
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
//
//fn pad_or_truncate<S: AsRef<str>>(s: S, width: usize, right_align: bool) -> String {
//    let s: &str = s.as_ref();
//    let ansi_regex = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
//    let plain_text = ansi_regex.replace_all(s, "").to_string();
//    let display_width = UnicodeWidthStr::width(plain_text.as_str());
//
//    let truncated = if display_width > width {
//        plain_text.chars().take(width).collect::<String>()
//    } else {
//        plain_text.clone()
//    };
//
//    let padding = " ".repeat(width.saturating_sub(display_width));
//    let padded_text = if right_align {
//        format!("{}{}", padding, truncated)
//    } else {
//        format!("{}{}", truncated, padding)
//    };
//
//    // Reapply ANSI codes to the padded/truncated text
//    s.replace(&plain_text, &padded_text)
//}

//fn pad_or_truncate<S: AsRef<str>>(s: S, width: usize, right_align: bool) -> String {
//    let s: &str = s.as_ref();
//
//    // This is where we’ll accumulate the truncated string with ANSI codes preserved
//    let mut truncated = String::new();
//    let mut visible_width = 0;
//
//    let mut iter = s.chars().peekable();
//    while let Some(ch) = iter.next() {
//        // Check if the character starts an ANSI sequence
//        if ch == '\x1b' && iter.peek() == Some(&'[') {
//            // Collect the entire ANSI sequence
//            let mut ansi_sequence = String::from(ch);
//            ansi_sequence.push(iter.next().unwrap()); // Consume '['
//
//            // Add the rest of the ANSI sequence characters
//            while let Some(&next_ch) = iter.peek() {
//                ansi_sequence.push(iter.next().unwrap());
//                if next_ch == 'm' {
//                    break;
//                }
//            }
//
//            // Push the entire ANSI sequence to the result
//            truncated.push_str(&ansi_sequence);
//        } else {
//            // Calculate the width of this character
//            let ch_width = UnicodeWidthStr::width(ch.to_string().as_str());
//
//            // Only add the character if it won't exceed the width limit
//            if visible_width + ch_width > width {
//                break;
//            }
//
//            // Append the character and update visible width
//            truncated.push(ch);
//            visible_width += ch_width;
//        }
//    }
//
//    // Add padding if needed
//    let padding = " ".repeat(width.saturating_sub(visible_width));
//    if right_align {
//        format!("{}{}", padding, truncated)
//    } else {
//        format!("{}{}", truncated, padding)
//    }
//}
