use std::path::{Path, PathBuf};
use dirs::home_dir;
use anyhow::{Result, Context};

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
/// * `Err(anyhow::Error)` if:
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
                .context("Could not determine base path")?; // Use anyhow for error context

            // Ensure that sub_path is provided
            let p = sub_path.context("Subpath must be provided")?; // Use anyhow for error context

            // Combine base path with provided sub_path
            Ok(base_path.join(p))
        }
    }
}