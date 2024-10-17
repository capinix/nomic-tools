use dirs::home_dir;
use eyre::{ContextCompat, eyre, Result};
use regex::Regex;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

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
/// Retrieves a full file path based on the provided options.
///
/// This function checks if a specific file path is given. If so, it returns that path.
/// If no file path is provided, it tries to determine a full path to the file by combining a 
/// base_path with a sub_path. If the base_path is not provided, the user's home directory 
/// is used as the base_path and combined with the sub_path to form the full file path.
///
/// # Parameters
///
/// * `file`: An optional path to a specific file. If provided, this path is returned directly.
/// * `base_path`: An optional base path. If this is not provided, the function will use the user's home directory.
/// * `sub_path`: An optional subpath that will be combined with the base path. This must be provided.
///
/// # Returns
///
/// * `Ok(PathBuf)` if a valid file path is successfully constructed.
/// * `Err(eyre::Error)` if:
///   - A base path cannot be determined.
///   - The subpath is not provided.
///
/// # Example
///
/// ```
/// let file_path = get_file(None, None, Some(Path::new("myfile.txt")))?
/// ```
pub fn get_file(
    file: Option<&Path>,
    base_path: Option<&Path>,
    sub_path: Option<&Path>,
) -> Result<PathBuf> {
    match file {
        Some(file_path) => Ok(file_path.to_path_buf()), // Return the provided file path
        None => {
            // Determine base_path
            let base_path = base_path
                .map(PathBuf::from) // Convert Option<&Path> to Option<PathBuf>
                .or_else(|| {
                    // Use dirs::home_dir() directly since it returns a PathBuf
                    home_dir()
                })
                .wrap_err("Could not determine base path")?; // Use eyre for error context

            // Ensure that sub_path is provided
            let p = sub_path.wrap_err("Subpath must be provided")?; // Use eyre for error context

            // Combine base path with provided sub_path
            Ok(base_path.join(p))
        }
    }
}

/// Resolves file and home options with mutual exclusivity.
/// Prioritizes subcommand-level options over the top-level options.
pub fn resolve_file_home(
    cmd_file: Option<PathBuf>, cmd_home: Option<PathBuf>, 
    global_file: Option<PathBuf>, global_home: Option<PathBuf>
) -> Result<(Option<PathBuf>, Option<PathBuf>)> {

    // Subcommand options take precedence over the top-level options.
    let file = cmd_file.or(global_file);
    let home = cmd_home.or(global_home);

    // Check mutual exclusivity of the resolved options.
    if file.is_some() && home.is_some() {
        return Err(eyre!("You cannot provide both --file and --home at the same time."));
    }

    Ok((file, home))
}

pub fn to_bool(val: String) -> Option<bool> {
    match val.trim().to_lowercase().as_str() {
        "true" | "yes" | "y" | "1" => Some(true),
        "false" | "no" | "n" | "0" => Some(false),
        "" => Some(false), // Treat empty string as false
        _ => None, // Invalid value, return None
    }
}

pub fn to_bool_string(val: String) -> Option<String> {
    match val.trim().to_lowercase().as_str() {
        "true" | "yes" | "y" | "1" => Some("true".to_string()),
        "false" | "no" | "n" | "0" => Some("false".to_string()),
        "" => Some("false".to_string()), // Handle empty string as "false"
        _ => None, // Invalid value, return None
    }
}

/// for clap
pub fn validate_ratio(value: &str) -> Result<f64, String> {
    match value.parse::<f64>() {
        Ok(val) if val >= 0.0 && val <= 1.0 => Ok(val),
        Ok(_) => Err(String::from("The minimum balance ratio must be between 0 and 1")),
        Err(_) => Err(String::from("Invalid input: please provide a valid number")),
    }
}

pub fn prompt_user(question: &str) -> io::Result<String> {
    print!("{} [y/N]: ", question);      // Print the question
    io::stdout().flush()?;               // Ensure the prompt is displayed immediately
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;  // Read user input from stdin
    Ok(input.trim().to_string())         // Return trimmed input
}

/// Searches for a configuration variable in the provided configuration content.
///
/// # Parameters
/// - `variable_name`: The name of the variable to search for (case-sensitive).
/// - `config_content`: The configuration content to search within.
///
/// # Returns
/// - `Option<String>`: Returns `Some(String)` if the value is found; otherwise `None`.
///
/// # Example
/// The following examples demonstrate the function's behavior with different configurations:
///
/// ```rust
/// let content = r#"
/// variable1 = "some value1"
/// variable2 = value2
/// variable3=value3
/// variable4 "some value4"
/// variable5 value5
/// "#;
///
/// assert_eq!(grep_config("variable1",   content), Some("some value1".to_string()));
/// assert_eq!(grep_config("variable2",   content), Some("value2".to_string()));
/// assert_eq!(grep_config("variable3",   content), Some("value3".to_string()));
/// assert_eq!(grep_config("variable4",   content), Some("some value4".to_string()));
/// assert_eq!(grep_config("variable5",   content), Some("value5".to_string()));
/// assert_eq!(grep_config("nonexistent", content), None);
/// ```
pub fn grep_config(variable_name: &str, config_content: &str) -> Option<String> {
    // Construct the regex pattern to capture various types of config values
    let regex = Regex::new(&(
        format!(r"(?m)^[[:space:]]*{variable_name}") +   // Match the variable name
        r"(?:[[:space:]]*=[[:space:]]*|[[:space:]]+)" +  // Match '=' or whitespace
        r#"(?:(?P<quote1>"[^"]*")|"# +                   // Match quoted values (")
        r"(?P<quote2>'[^']*')|" +                        // Match quoted values (')
        r"(?P<unquoted>[^[:space:]]+)).*$"               // Match unquoted values
    )).unwrap();

    // Capture the value if it matches the pattern
    regex.captures(config_content).and_then(|captures| {
        // Return the first matching capture group (quoted or unquoted)
        captures.get(1).or_else(|| captures.get(2)).or_else(|| captures.get(3)).map(|value| {
            // Trim surrounding quotes if present and return as String
            value.as_str().trim_matches('"').trim_matches('\'').to_string()
        })
    })
}
