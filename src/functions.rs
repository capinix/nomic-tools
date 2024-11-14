//use dirs::home_dir;
use eyre::Result;
use regex::Regex;
use std::io::{self, Read, Write};
use std::str::FromStr;
use num_format::{Locale, ToFormattedString};
//use console::strip_ansi_codes;
use tabled::Tabled;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use unicode_width::UnicodeWidthStr;

fn pluralize<S: AsRef<str>>(word: S, count: u64) -> String {
    let word: &str = word.as_ref();
    if count == 1 {
        word.to_string()
    } else if word.ends_with("y") && matches!(word.chars().nth_back(1), Some(c) if !"aeiou".contains(c)) {
        format!("{}ies", &word[..word.len() - 1])
    } else if word.ends_with("ch") || word.ends_with("s") || word.ends_with("sh") || word.ends_with("x") || word.ends_with("z") {
        format!("{}es", word)
    } else if word.ends_with("f") {
        format!("{}ves", &word[..word.len() - 1])
    } else if word.ends_with("fe") {
        format!("{}ves", &word[..word.len() - 2])
    } else {
        format!("{}s", word)
    }
}

pub fn format_date_offset(seconds: u64) -> String {
    let now = chrono::Local::now();
    let then = now + chrono::Duration::seconds(seconds as i64);

    if now.date_naive() == then.date_naive() {
        // Same day, show only the time
        then.format("%H:%M").to_string()
    } else {
        // Different day, show full date and time
        then.format("%A, %Y-%m-%d %H:%M").to_string()
    }
}


pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        if hours > 0 && minutes > 0 {
            format!(
                "{} {}, {} {}, {} {}",
                days, pluralize("day", days),
                hours, pluralize("hour", hours),
                minutes, pluralize("minute", minutes)
            )
        } else if hours > 0 {
            format!(
                "{} {}, {} {}",
                days, pluralize("day", days),
                hours, pluralize("hour", hours)
            )
        } else if minutes > 0 {
            format!(
                "{} {}, {} {}",
                days, pluralize("day", days),
                minutes, pluralize("minute", minutes)
            )
        } else {
            format!("{} {}", days, pluralize("day", days))
        }
    } else if hours > 0 {
        if minutes > 0 {
            format!(
                "{} {}, {} {}",
                hours, pluralize("hour", hours),
                minutes, pluralize("minute", minutes)
            )
        } else {
            format!("{} {}", hours, pluralize("hour", hours))
        }
    } else {
        format!("{} {}", minutes, pluralize("minute", minutes))
    }
}

pub fn truncate_with_ellipsis<S: AsRef<str>>(input: S, max_len: usize) -> String {
    let input: &str = input.as_ref();
    if input.chars().count() > max_len {
        input.chars().take(max_len - 3).collect::<String>() + "..."
    } else {
        input.to_string()
    }
}

/// Reads data from standard input (stdin) with a specified number of attempts
/// and a timeout duration.
///
/// This function attempts to read bytes from stdin up to a maximum number of
/// attempts. If the read operation does not succeed within the specified timeout,
/// an error is returned. The data is returned as a `Vec<u8>`.
///
/// # Parameters
///
/// - `max_attempts`: The maximum number of attempts to read from stdin before
///   giving up. If all attempts fail, an error is returned.
/// - `timeout_in_seconds`: The duration in seconds to wait for data on each
///   attempt before timing out. This value is specified as a `u64`.
///
/// # Returns
///
/// Returns a `Result<Vec<u8>>`, which will contain the read data as a vector
/// of bytes on success, or an error if the read operation fails after the
/// maximum number of attempts or due to an unexpected error.
///
/// # Examples
///
/// ```rust
/// let data = read_stdin(3, 10)?;
/// // This will attempt to read data from stdin with a 10-second timeout,
/// // retrying up to 3 times if necessary.
/// ```
///
/// # Errors
///
/// The function may return an error if it fails to read from stdin after the
/// specified number of attempts, or if there is an unexpected error during
/// the read operation.
pub fn read_stdin(max_attempts: usize, timeout_in_seconds: u64) -> Result<Vec<u8>> {
    let stdin = Arc::new(Mutex::new(io::stdin()));
    let (sender, receiver) = std::sync::mpsc::channel();

    for attempt in 0..max_attempts {
        let stdin_clone = Arc::clone(&stdin);
        let sender_clone = sender.clone();

        // Spawn a new thread to read from stdin
        thread::spawn(move || {
            let mut handle = stdin_clone.lock().unwrap();
            let mut local_buf = Vec::new();

            // Attempt to read from stdin
            if handle.read_to_end(&mut local_buf).is_ok() {
                // Send the buffer only if it contains data
                if !local_buf.is_empty() {
                    sender_clone.send(local_buf).unwrap();
                }
            }
        });

        // Wait for data or timeout
        let deadline = Instant::now() + Duration::from_secs(timeout_in_seconds);
        while Instant::now() < deadline {
            if let Ok(data) = receiver.recv_timeout(Duration::from_secs(1)) {
                // Return the received data as Vec<u8>
                return Ok(data);
            }
        }

        // If this was the last attempt, return an error
        if attempt == max_attempts - 1 {
            return Err(eyre::eyre!(
                "Timeout: Failed to read input from stdin after {} attempts",
                max_attempts
            ));
        }

        // Optionally sleep before the next attempt
        thread::sleep(Duration::from_millis(100));
    }

    Err(eyre::eyre!("Timeout: Unexpected error in read_stdin"))
}

/// A struct for formatting a number with configurable scale, decimal places, trimming options,
/// and an optional threshold for when to display decimals.
pub struct NumberDisplay {
    /// The integer number to be formatted.
    number: u64,

    /// Number of digits to consider as the fractional part when displaying the number.
    /// For example, if `scale` is set to 6, the last 6 digits of `number` will be treated
    /// as the decimal part.
    scale: usize,

    /// The number of decimal places to display. Excess decimals will be truncated, and
    /// if there are fewer decimals than this value, zeros will be appended unless trimming is enabled.
    decimal_places: usize,

    /// If `true`, trailing zeros in the decimal part are removed.
    /// For instance, "6.5000" would become "6.5".
    trim: bool,

    /// A threshold for the integer part above which the decimal portion is omitted.
    /// For example, if `integer_threshold` is set to 10 and the integer part is greater than 10,
    /// only the integer part will be displayed without decimals.
    integer_threshold: u64,
}

impl NumberDisplay {
    /// Creates a new `NumberDisplay` instance with default settings for scale, decimal places, trimming, 
    /// and threshold. The number to format must be specified on instantiation.
    ///
    /// # Parameters
    /// - `number`: The integer number to be formatted.
    ///
    /// # Default Settings
    /// - `scale`: 6 (scales the number as if it had 6 decimal places)
    /// - `decimal_places`: 2 (limits the displayed decimal places)
    /// - `trim`: false (does not remove trailing zeros by default)
    /// - `integer_threshold`: `u64::MAX` (displays decimals for all values by default)
    pub fn new(number: u64) -> Self {
        Self {
            number,
            scale: 6,                    // Default scale
            decimal_places: 2,           // Default decimal places
            trim: false,                 // Default no trimming
            integer_threshold: u64::MAX, // Default threshold high enough to include decimals by default
        }
    }

    /// Sets the scale for the number display, which adjusts the position of the decimal point.
    ///
    /// # Parameters
    /// - `scale`: The number of digits to treat as the fractional part (i.e., the decimal scale).
    pub fn scale(mut self, scale: usize) -> Self {
        self.scale = scale;
        self
    }

    /// Sets the number of decimal places to display.
    ///
    /// # Parameters
    /// - `decimal_places`: The number of digits after the decimal point.
    pub fn decimal_places(mut self, decimal_places: usize) -> Self {
        self.decimal_places = decimal_places;
        self
    }

    /// Sets whether trailing zeros should be removed from the decimal part.
    ///
    /// # Parameters
    /// - `trim`: If true, trailing zeros and unnecessary decimal points are removed.
    pub fn trim(mut self, trim: bool) -> Self {
        self.trim = trim;
        self
    }

    /// Sets a threshold for the integer part above which decimal places are omitted.
    ///
    /// # Parameters
    /// - `threshold`: The maximum integer value for which decimals should still be shown.
    pub fn integer_threshold(mut self, threshold: u64) -> Self {
        self.integer_threshold = threshold;
        self
    }

    /// Formats the number according to the specified settings for scale, decimal places, trimming,
    /// and integer threshold.
    ///
    /// # Returns
    /// - A `String` representing the formatted number.
    ///
    /// # Example
    /// ```
    /// let formatted = NumberDisplay::new(123456)
    ///     .scale(6)
    ///     .decimal_places(2)
    ///     .trim(true)
    ///     .integer_threshold(10)
    ///     .format();
    /// ```
    pub fn format(&self) -> String {
        // Pad the number to ensure it has at least `scale + 1` digits.
        let padded = format!("{:0width$}", self.number, width = self.scale + 1);

        // Integer part (all but the last `scale` characters).
        let integer_part = &padded[..padded.len().saturating_sub(self.scale)];

        // Convert integer_part to u64 for formatting with thousands separators
        let formatted_integer = match u64::from_str(integer_part) {
            Ok(num) => num.to_formatted_string(&Locale::en),
            Err(_) => "0".to_string(), // Fallback in case conversion fails
        };

        // Decimal part is the last `scale` characters.
        let decimal_part = &padded[padded.len().saturating_sub(self.scale)..];

        // Adjust the decimal part to match the specified decimal places.
        let truncated_decimal = if decimal_part.len() > self.decimal_places {
            &decimal_part[..self.decimal_places]
        } else {
            &format!("{:0<width$}", decimal_part, width = self.decimal_places)
        };

        // Determine if the decimal part should be included based on integer_threshold.
        let should_include_decimals = match u64::from_str(integer_part) {
            Ok(num) => num <= self.integer_threshold,
            Err(_) => true, // Default to including decimals if parsing fails
        };

        if self.decimal_places == 0 || !should_include_decimals {
            // Skip decimals if decimal_places is zero or if integer part exceeds threshold.
            formatted_integer
        } else {
            let mut formatted_number = format!("{}.{}", formatted_integer, truncated_decimal);

            // Apply trimming if specified.
            if self.trim {
                formatted_number = formatted_number.trim_end_matches('0').trim_end_matches('.').to_string();
            }
            formatted_number
        }
    }
}






/// A struct representing a row in a table with up to 10 columns.
///
/// This struct is derived from `Tabled`, which allows it to be easily displayed
/// in a tabular format. Each cell in the table is represented by a `String`.
#[derive(Clone, Tabled)]
pub struct TableColumns {
    pub cell0: String,
    pub cell1: String,
    pub cell2: String,
    pub cell3: String,
    pub cell4: String,
    pub cell5: String,
    pub cell6: String,
    pub cell7: String,
    pub cell8: String,
    pub cell9: String,
}

impl TableColumns {
    /// Creates a new `TableColumns` instance from a vector of string slices.
    ///
    /// If the input vector has fewer than 10 items, the remaining cells will
    /// be filled with empty strings (`String::new()`). If the input vector has
    /// more than 10 items, the extra items will be ignored.
    ///
    /// # Arguments
    ///
    /// * `input` - A vector of string slices that represent the initial cell values.
    ///
    /// # Examples
    ///
    /// ```
    /// let row = TableColumns::new(vec!["Alice", "Validator 1", "100", "200", "0.5"]);
    /// assert_eq!(row.cell0, "Alice");
    /// assert_eq!(row.cell1, "Validator 1");
    /// assert_eq!(row.cell2, "100");
    /// assert_eq!(row.cell3, "200");
    /// assert_eq!(row.cell4, "0.5");
    /// assert_eq!(row.cell5, ""); // cell5 is empty
    /// ```
    ///
    /// # Assertions
    ///
    /// - Ensures that the output has exactly 10 cells.
    /// - Fills any missing cells with empty strings.
    pub fn new(input: Vec<&str>) -> Self {
        // Map input items to String, and pad with String::new() if there are fewer than 10 items
        let mut cells = input.into_iter().map(|s| s.to_string()).collect::<Vec<String>>();

        // Pad to ensure exactly 10 items
        cells.resize(10, String::new());

        // Return a new TableColumns instance with the prepared cells
        TableColumns {
            cell0: cells[0].clone(),
            cell1: cells[1].clone(),
            cell2: cells[2].clone(),
            cell3: cells[3].clone(),
            cell4: cells[4].clone(),
            cell5: cells[5].clone(),
            cell6: cells[6].clone(),
            cell7: cells[7].clone(),
            cell8: cells[8].clone(),
            cell9: cells[9].clone(),
        }
    }
}

pub fn pad_or_truncate<S: AsRef<str>>(s: S, width: usize, right_align: bool) -> String {
    let s: &str = s.as_ref();

    // This is where we’ll accumulate the truncated string with ANSI codes preserved
    let mut truncated = String::new();
    let mut visible_width = 0;

    let mut iter = s.chars().peekable();
    while let Some(ch) = iter.next() {
        // Check if the character starts an ANSI sequence
        if ch == '\x1b' && iter.peek() == Some(&'[') {
            // Collect the entire ANSI sequence
            let mut ansi_sequence = String::from(ch);
            ansi_sequence.push(iter.next().unwrap()); // Consume '['

            // Add the rest of the ANSI sequence characters
            while let Some(&next_ch) = iter.peek() {
                ansi_sequence.push(iter.next().unwrap());
                if next_ch == 'm' {
                    break;
                }
            }

            // Push the entire ANSI sequence to the result
            truncated.push_str(&ansi_sequence);
        } else {
            // Calculate the width of this character using UnicodeWidthStr
            let ch_width = UnicodeWidthStr::width(ch.to_string().as_str());

            // If this character is an emoji or wide character, ensure its width is treated as 1
            let adjusted_ch_width = if ch_width > 1 { 1 } else { ch_width };

            // Only add the character if it won't exceed the width limit
            if visible_width + adjusted_ch_width > width {
                break;
            }

            // Append the character and update visible width
            truncated.push(ch);
            visible_width += adjusted_ch_width;
        }
    }

    // Add padding if needed
    let padding = " ".repeat(width.saturating_sub(visible_width));
    if right_align {
        format!("{}{}", padding, truncated)
    } else {
        format!("{}{}", truncated, padding)
    }
}

//pub fn pad_or_truncate(s: &str, width: usize, right_align: bool) -> String {
//    // Calculate length without ANSI codes
//    let stripped_len = strip_ansi_codes(s).chars().count();
//
//    if stripped_len > width {
//        // Truncate while keeping ANSI codes intact
//        let mut visible_count = 0;
//        let truncated: String = s.chars()
//            .take_while(|&c| {
//                // Only count non-ANSI characters toward the width limit
//                if !c.is_ascii_control() {
//                    visible_count += 1;
//                }
//                visible_count <= width
//            })
//            .collect();
//        truncated
//    } else {
//        // Add padding, keeping ANSI codes intact
//        let padding = " ".repeat(width - stripped_len);
//        if right_align {
//            format!("{}{}", padding, s) // Right-align
//        } else {
//            format!("{}{}", s, padding) // Left-align
//        }
//    }
//}

pub fn format_to_millions(value: u64, decimal_places: Option<usize>) -> String {
    let integer_part = value / 1_000_000;
    let decimal_part = value % 1_000_000;

    // Format the integer part with a thousands separator
    let formatted_integer = integer_part.to_formatted_string(&Locale::en);

    match decimal_places {
        Some(places) if decimal_part > 0 => {
            // Format the decimal part with 6 digits and pad/truncate to required places
            let decimal_str = pad_or_truncate(&format!("{:06}", decimal_part), places, false);
            format!("{}.{}", formatted_integer, decimal_str)
                .trim_end_matches('.')
                .to_string()
        }
        None if decimal_part > 0 => {
            // Trim trailing zeros dynamically when decimal places are unspecified
            format!("{}.{}", formatted_integer, format!("{:06}", decimal_part))
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        }
        _ => formatted_integer, // No decimals needed
    }
}

#[derive(Clone)]
pub enum TaskStatus {
    Done,    // ✅
    NotDone, // ❌
}

impl TaskStatus {
    pub fn to_symbol(&self) -> &'static str {
        match self {
            TaskStatus::Done    => "✅",
            TaskStatus::NotDone => "❌",
        }
    }

    // Convert from a boolean to TaskStatus
    pub fn from_bool(value: bool) -> Self {
        if value {
            TaskStatus::Done
        } else {
            TaskStatus::NotDone
        }
    }

    // Convert from TaskStatus to boolean
    // we will use this whn reading the logs
    #[allow(dead_code)]
    pub fn to_bool(&self) -> bool {
        match self {
            TaskStatus::Done => true,
            TaskStatus::NotDone => false,
        }
    }
}


/// Validates whether a given string is a valid Nomic Bech32 address.
///
/// This function checks if the provided address starts with `nomic1` 
/// and is followed by exactly 38 alphanumeric lowercase characters (a-z, 0-9).
/// The address is converted to lowercase before validation to ensure case insensitivity.
///
/// # Parameters
/// - `address`: A string slice representing the address to validate.
///
/// # Returns
/// - `true` if the address is valid according to the specified pattern; otherwise, `false`.
pub fn is_valid_nomic_address(address: &str) -> bool {
    let re = Regex::new(r"^nomic1[a-z0-9]{38}$").unwrap();
    re.is_match(&address.to_lowercase())
}

pub fn validate_positive<T>(value: &str) -> Result<T, String>
where
    T: FromStr + PartialOrd + std::fmt::Display,
    T::Err: std::fmt::Display,
{
    match value.parse::<T>() {
        Ok(v) => {
            if let Ok(zero) = T::from_str("0") {
                if v > zero {
                    Ok(v)
                } else {
                    Err(format!("Value must be greater than 0, but got {}", v))
                }
            } else {
                Err("Unable to parse '0' for comparison".to_string())
            }
        }
        Err(e) => Err(format!("Invalid number: {}", e)),
    }
}



#[allow(dead_code)]
pub fn to_bool<S: AsRef<str>>(val: S) -> Option<bool> {
    let val: &str = val.as_ref();
    match val.trim().to_lowercase().as_str() {
        "true" | "yes" | "y" | "1" => Some(true),
        "false" | "no" | "n" | "0" => Some(false),
        "" => Some(false), // Treat empty string as false
        _ => None, // Invalid value, return None
    }
}

//pub fn to_bool_string(val: String) -> Option<String> {
//    match val.trim().to_lowercase().as_str() {
//        "true" | "yes" | "y" | "1" => Some("true".to_string()),
//        "false" | "no" | "n" | "0" => Some("false".to_string()),
//        "" => Some("false".to_string()), // Handle empty string as "false"
//        _ => None, // Invalid value, return None
//    }
//}

/// for clap
pub fn validate_ratio(value: &str) -> Result<f64, String> {
    //let value: &str = value.as_ref();
    match value.parse::<f64>() {
        Ok(val) if val >= 0.0 && val <= 1.0 => Ok(val),
        Ok(_) => Err(String::from("The minimum balance ratio must be between 0 and 1")),
        Err(_) => Err(String::from("Invalid input: please provide a valid number")),
    }
}

pub fn prompt_user<S: AsRef<str>>(question: S) -> io::Result<String> {
    let question: &str = question.as_ref();
    print!("{} [y/N]: ", question);      // Print the question
    io::stdout().flush()?;               // Ensure the prompt is displayed immediately
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;  // Read user input from stdin
    Ok(input.trim().to_string())         // Return trimmed input
}

