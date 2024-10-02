use std::path::{Path, PathBuf};
use dirs::home_dir;
use eyre::{ContextCompat, eyre, Result};

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

/// Checks if a given string is a valid hexadecimal string.
///
/// A valid hexadecimal string consists of characters 0-9 and a-f (case insensitive).
/// Additionally, it must have an even length to ensure that each byte is represented
/// by two hexadecimal characters.
///
/// # Arguments
///
/// * `s` - A string slice that is validated as a hexadecimal string.
///
/// # Returns
///
/// * `Ok(s)` if the string is a valid hexadecimal representation.
/// * `Err(eyre::Error)` if the string is not valid or empty.
///
/// # Example
///
/// ```
/// let valid_hex = is_hex("1a2b3c").unwrap();
/// let invalid_hex = is_hex("1g2h3i").is_err(); // Invalid hex
/// let odd_length = is_hex("123").is_err();	// Odd length
/// ```
pub fn is_hex(s: &str) -> Result<String> {
	if s.is_empty() {
		return Err(eyre::eyre!("Input string cannot be empty"));
	}
	if s.len() % 2 != 0 || !s.chars().all(|c| c.is_digit(16)) {
		return Err(eyre::eyre!("Invalid hexadecimal string"));
	}
	Ok(s.to_string())
}
