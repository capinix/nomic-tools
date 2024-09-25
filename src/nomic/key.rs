use hex::decode;
//use hex::FromHexError;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
//use hex::FromHex;

pub fn set_key(file: &Path, hex_str: &str) -> Result<(), Box<dyn Error>> {
    // Validate and decode the hex string into binary data
    let binary_data = match decode(hex_str) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Invalid hex string: {}", e);
            return Err(Box::new(e));
        }
    };

    // Open the file for writing (create or overwrite)
    let mut file = File::create(file)?;

    // Write the binary data to the file
    file.write_all(&binary_data)?;

    // Flush the output to ensure all data is written
    file.flush()?;

    Ok(())
}

pub fn get_key(file: &Path) -> Result<String, Box<dyn Error>> {
    // Open the file for reading
    let mut file = File::open(file)?;
    
    // Create a buffer to hold the binary data
    let mut buffer = Vec::new();

    // Read the entire file into the buffer
    file.read_to_end(&mut buffer)?;

    // Convert the binary data to a hexadecimal string
    let hex_value = hex::encode(&buffer);

    // Return the hexadecimal string
    Ok(hex_value)
}
