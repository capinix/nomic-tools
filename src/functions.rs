use dirs::home_dir;
use eyre::{ContextCompat, eyre, Result};
use regex::Regex;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use crate::globals::PROFILES_DIR;

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

///// Resolves file and home options with mutual exclusivity.
///// Prioritizes subcommand-level options over the top-level options.
//pub fn resolve_file_home(
//    cmd_file: Option<PathBuf>, cmd_home: Option<PathBuf>, 
//    global_file: Option<PathBuf>, global_home: Option<PathBuf>
//) -> Result<(Option<PathBuf>, Option<PathBuf>)> {
//
//    // Subcommand options take precedence over the top-level options.
//    let file = cmd_file.or(global_file);
//    let home = cmd_home.or(global_home);
//
//    // Check mutual exclusivity of the resolved options.
//    if file.is_some() && home.is_some() {
//        return Err(eyre!("You cannot provide both --file and --home at the same time."));
//    }
//
//    Ok((file, home))
//}

///// Returns a path given an optional profile, home, or file, and an ending path.
///// 
///// The structure is assumed as follows:
///// - `file` is prioritized if provided.
///// - `home` or `profile` is used to construct the path if `file` is not provided.
///// - `end` must be provided when using `profile` or `home`.
/////
///// Example:
/////     - `Option<home> = Some("/home/user/Documents/profile_name")`
/////     - `Option<profile> = Some("profile_name")`
/////     - `Option<file> = Some("/home/user/Documents/profile_name/ending/path.file")`
/////
/////     `path_end = "ending/path.file"`
//pub fn profile_home_file(
//    profile: Option<&str>,
//    home: Option<&Path>, 
//    file: Option<&Path>,
//    end: Option<&Path>,
//) -> Result<PathBuf> {
//    // If all are None, check end
//    if profile.is_none() && home.is_none() && file.is_none() {
//        if end.is_none() {
//            return Err(eyre!("No arguments specified"));
//        } else {
//            // Attempt to get the home directory
//            match home::home_dir() {
//                Some(path) if !path.as_os_str().is_empty() => {
//                    return Ok(path.join(end.unwrap())); // Join home with end
//                }
//                _ => return Err(eyre!("Unable to detect home directory")),
//            }
//        }
//    }
//
//    // If file is provided, return it directly
//    if let Some(file) = file {
//        return Ok(file.to_path_buf());
//    }
//
//    // If end is not provided, we need to check for home or profile
//    let end = end.ok_or_else(|| eyre!("If no file is provided, an end path must be given."))?;
//
//    // Check for home directory
//    if let Some(home) = home {
//        return Ok(home.join(end)); // Join the home path with the end path
//    }
//
//    // If profile is provided, construct the path using PROFILES_DIR
//    if let Some(profile) = profile {
//        return Ok(PROFILES_DIR.join(profile).join(end));
//    }
//
//    // If none of the above conditions matched, return an error
//    Err(eyre!("Insufficient arguments to construct a path."))
//}

/// Constructs a file path based on the provided input and relative path.
///
/// This function first checks if the `input` string represents a valid file or directory.
/// - If `input` is a valid file path, it returns the path as a `PathBuf`.
/// - If `input` is a directory, it combines it with the `sub_path` to create the full path.
/// - If `input` does not correspond to a file or directory, it treats it as a profile name
///   and combines it with the `PROFILES_DIR` and `sub_path`.
/// - If no `input` is provided, it attempts to use the user's home directory,
///   combining it with the `sub_path`.
///
/// # Arguments
/// 
/// * `input` - An optional string slice that represents the input path or profile name.
/// * `sub_path` - An optional reference to a `Path` that is used to extend the input path.
///
/// # Returns
/// 
/// * `Result<PathBuf>` - Returns the constructed path as a `PathBuf` if successful, or an error if any checks fail.
///
/// # Errors
/// 
/// This function returns an error if:
/// - The `input` string cannot be converted to a valid path.
/// - The `sub_path` is required but not provided when dealing with directories or profiles.
/// - The home directory cannot be detected.
///
/// # Example
///
/// ```
/// let path = construct_path(Some("example_profile"), Some(Path::new("subdir/file.txt")));
/// ```
/// 
pub fn construct_path(
    input: Option<&str>,
    sub_path: Option<&Path>,
) -> Result<PathBuf> {
    // Check if input is provided
    if let Some(input_str) = input {
        // Create a Path from the input string
        let input_path = Path::new(input_str);

        // Check if it's a file
        if input_path.is_file() {
            return Ok(input_path.to_path_buf());
        }

        // Check if it's a directory
        if input_path.is_dir() {
            let end_path = sub_path.ok_or_else(|| eyre!(
                "Relative path must be provided if input is a directory."
            ))?;
            return Ok(input_path.join(end_path));
        }

        // Handle profile (assume it's a profile name)
        let end_path = sub_path.ok_or_else(|| eyre!(
            "Relative path must be provided if input is a profile."
        ))?;
        return Ok(PROFILES_DIR.join(input_path).join(end_path));

    } else {
        // If no input is provided, try to get home directory
        let home_dir = home::home_dir().ok_or_else(|| eyre!(
            "Unable to detect home directory."
        ))?;
        let end_path = sub_path.ok_or_else(|| eyre!(
            "Relative path must be provided when using home directory."
        ))?;
        return Ok(home_dir.join(end_path));
    }
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
