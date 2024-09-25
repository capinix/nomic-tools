use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn get_nonce(nonce_file: &Path) -> u64 {
    // Attempt to open the file
    let mut file = match File::open(nonce_file) {
        Ok(f) => f,
        Err(e) => {
            // Check if the error is due to the file not existing
            if e.kind() == io::ErrorKind::NotFound {
                eprintln!("File '{}' does not exist. Creating a new file.", nonce_file.display());

                // Create a new file and write 0 to it
                match File::create(nonce_file) {
                    Ok(mut new_file) => {
                        // Write the binary representation of 0
                        let zero_bytes = (0u64).to_be_bytes();
                        if let Err(e) = new_file.write_all(&zero_bytes) {
                            eprintln!("Error writing to file '{}': {}", nonce_file.display(), e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error creating file '{}': {}", nonce_file.display(), e);
                        return 0; // Return 0 on error
                    }
                }
                return 0; // Return 0 since the file didn't exist
            } else {
                eprintln!("Error opening file '{}': {}", nonce_file.display(), e);
                return 0; // Return 0 on other errors
            }
        }
    };

    let mut input = Vec::new();

    // Read the binary contents of the file into a buffer
    if let Err(e) = file.read_to_end(&mut input) {
        eprintln!("Error reading file '{}': {}", nonce_file.display(), e);
        return 0; // Return 0 on error
    }

    // Check if the input size is within the u64 limit (8 bytes)
    if input.len() <= 8 {
        let mut bytes = [0u8; 8]; // Create an array to hold the u64 bytes
        bytes[..input.len()].copy_from_slice(&input); // Copy input bytes to the array
        return u64::from_be_bytes(bytes); // Convert to u64
    }

    // If the input is too large, return 0
    eprintln!("File content too large to fit in u64: {}", nonce_file.display());
    0
}

pub fn set_nonce(value: u64, nonce_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create or open the file for writing
    let mut file = File::create(nonce_file)
        .map_err(|e| {
            eprintln!("Error creating or opening file '{}': {}", nonce_file.display(), e);
            e
        })?;

    // Convert the value to binary and write to the file
    let bytes = value.to_be_bytes();
    file.write_all(&bytes)
        .map_err(|e| {
            eprintln!("Error writing to file '{}': {}", nonce_file.display(), e);
            e
        })?;

    Ok(())
}
