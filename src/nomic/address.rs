use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};

/// Converts a hexadecimal string input to binary data and writes it to an output destination.
///
/// # Arguments
///
/// * `input_source` - The input source, either a file path or stdin.
/// * `output_dest` - The output destination, either a file path or stdout.
///
/// # Errors
///
/// This function will return an error if reading the input or writing the output fails.
fn hex_to_bin(input_source: Option<&str>, output_dest: Option<&str>) -> Result<(), Box<dyn Error>> {
    // Determine input source
    let input: Box<dyn Read> = if let Some(input_file) = input_source {
        Box::new(BufReader::new(File::open(input_file)?))
    } else {
        // stdin is used if no input file is provided
        Box::new(io::stdin())
    };

    // Determine output destination
    let mut output: Box<dyn Write> = if let Some(output_file) = output_dest {
        Box::new(File::create(output_file)?)
    } else {
        Box::new(io::stdout())
    };

    // Read input
    let mut buffer = Vec::new();
    input.read_to_end(&mut buffer)?;

    // Handle empty stdin case
    if buffer.is_empty() && io::stdin().is_terminal() {
        return Err("No data provided on stdin".into());
    }

    // Convert hex string to binary data if not empty
    if !buffer.is_empty() {
        let hex_string = String::from_utf8(buffer)?;
        let binary_data = hex::decode(hex_string.trim())?;
        output.write_all(&binary_data)?;
    }

    Ok(())
}

