use std::io::{self, Read, Write};
use std::convert::TryInto;
use std::error::Error;

pub fn binary_to_decimal() -> Result<(), Box<dyn Error>> {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;

    if input.len() != 8 {
        return Err("Input size does not match u64".into());
    }

    let decimal_value = u64::from_be_bytes(
        input.try_into().map_err(|_| "Failed to convert input to u64")?
    );
    println!("{}", decimal_value);
    Ok(())
}

pub fn binary_to_hex() -> Result<(), Box<dyn Error>> {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;

    let hex_value = hex::encode(&input);
    println!("{}", hex_value);
    Ok(())
}

pub fn decimal_to_binary() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    let decimal_value: u64 = input.parse().map_err(|_| "Invalid decimal input")?;
    let hex_value = format!("{:016X}", decimal_value);
    
    let bytes = hex::decode(&hex_value).map_err(|_| "Hex decoding failed")?;
    io::stdout().write_all(&bytes)?;
    Ok(())
}

pub fn decimal_to_hex() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    let decimal_value: u64 = input.parse().map_err(|_| "Invalid decimal input")?;
    let hex_value = format!("{:X}", decimal_value);
    println!("{}", hex_value);
    Ok(())
}

pub fn hex_to_binary() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let input = input.trim().replace("\n", "");

    let bytes = hex::decode(&input).map_err(|_| "Invalid hex input")?;
    io::stdout().write_all(&bytes)?;
    Ok(())
}

pub fn hex_to_decimal() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let input = input.trim().replace("\n", "");

    let decimal_value = u64::from_str_radix(&input, 16).map_err(|_| "Invalid hex input")?;
    println!("{}", decimal_value);
    Ok(())
}
