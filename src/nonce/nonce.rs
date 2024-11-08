
use std::path::Path;
use eyre::Result;
use std::fs;
use eyre::eyre;
use log::info;
use crate::functions::read_stdin;

pub struct Nonce(u64);

impl Nonce {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    pub fn from_bytes(bytes: [u8; 8]) -> Self {
        let value = u64::from_le_bytes(bytes);
        Self(value)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P, dont_overwrite: bool) -> Result<()> {
        let path: &Path = path.as_ref();

       // Check if the file exists and dont_overwrite flag is true
       if dont_overwrite && path.exists() {
           return Err(eyre!("File already exists and overwriting is disabled"));
       }

        std::fs::write(path, self.to_bytes())?;
        Ok(())
    }

    pub fn from_vec<T: AsRef<Vec<u8>>>(data: T) -> Result<Self> {
        let buf = data.as_ref();

        // Check for empty input
        if buf.is_empty() {
            return Err(eyre::eyre!("Input data is empty"));
        }

        // Check if the input is a valid UTF-8 string and can be interpreted as a filename/path
        let buf_str = String::from_utf8_lossy(buf);
        let path = Path::new(buf_str.as_ref());

        // Try to read from the file if it exists
        if path.exists() {
            // Attempt to read the file
            match fs::read(path) {
                Ok(file_data) => {
                    // Ensure file data is not empty
                    if !file_data.is_empty() {
                        // Recursively apply checks to the file content
                        return Self::from_vec(file_data);
                    } else {
                        info!("File is empty: {:?}", path);
                        return Ok(Self::new(0));
                    }
                },
                Err(_) => {
                    // If the file cannot be read, return an error
                    return Err(eyre::eyre!("Cannot read file at path: {:?}", path));
                }
            }
        } else {
            info!("Input string is not a valid file path; checking for u64...");
        }

        // Check if the data length is exactly 8 bytes; if so, treat it as binary.
        if buf.len() == 8 {
            let mut binary_data = [0u8; 8];
            binary_data.copy_from_slice(buf);
            return Ok(Self::from_bytes(binary_data));
        }

        // Attempt to interpret the bytes as a UTF-8 string.
        if let Ok(text_data) = String::from_utf8(buf.to_vec()) {
            match text_data.trim().parse::<u64>() {
                Ok(parsed_value) => {
                    return Ok(Nonce::new(parsed_value));
                },
                Err(_) => {
                    info!("Input string is not a valid u64; checking for hex format...");
                }
            }
        } else {
            info!("Input data is not valid UTF-8.");
        }

        // At this point, all checks have failed, and we can handle it as desired
        Err(eyre::eyre!("Input could not be parsed as valid data."))
    }

    pub fn from_stdin(max_attempts: usize, timeout_in_seconds: u64) -> Result<Self> {
        let data_bytes = read_stdin(max_attempts, timeout_in_seconds)?;
        Self::from_vec(data_bytes)
    }

    pub fn load<S: AsRef<str>>(input: S) -> Result<Self> {
        let input_str: &str = input.as_ref();
        let input_vec = input_str.as_bytes().to_vec();
        Self::from_vec(input_vec)
    }
}

///// Retrieves the nonce file path.
/////
///// This helper function attempts to construct the path of the nonce file located in the `.orga-wallet`
///// directory. It utilizes the provided optional `file` and `home` parameters to determine the correct path.
/////
///// # Parameters
/////
///// - `file`: An optional path to a specific nonce file.
///// - `home`: An optional base path; if not provided, the user's home directory will be used.
/////
///// # Returns
/////
///// - `Ok(PathBuf)` containing the path to the nonce file if successful.
///// - `Err(anyhow::Error)` if there is an issue retrieving the nonce file path.
//fn get_nonce_file(file: Option<&Path>, home: Option<&Path>) -> Result<PathBuf> {
//    let sub_path = Path::new(".orga-wallet").join("nonce");
//    get_file(file, home, Some(&sub_path))
//}
//
///// Retrieves the nonce value from a binary file.
/////
///// This function attempts to read the contents of the specified nonce file and interpret it as a `u64` value.
///// If the file does not exist or cannot be read, it will return an error.
/////
///// # Parameters
/////
///// - `file`: An optional path to a specific nonce file.
///// - `home`: An optional base path; the home directory will be used if not provided.
/////
///// # Returns
/////
///// - `Ok(u64)` containing the nonce value if successfully retrieved.
///// - `Err(eyre::Error)` if an error occurs while retrieving the nonce file or reading its contents.
//pub fn export(file: Option<&Path>, home: Option<&Path>) -> Result<u64> {
//	let nonce_file = get_nonce_file(file, home)
//		.context("Failed to get nonce file path")?;
//
//	let mut file = File::open(&nonce_file)
//		.with_context(|| format!("Failed to open nonce file at {:?}", nonce_file))?;
//	
//	let mut input = Vec::new();
//	file.read_to_end(&mut input)
//		.with_context(|| format!("Failed to read from nonce file at {:?}", nonce_file))?;
//
//	if input.len() > 8 {
//		return Err(eyre::eyre!("File content too large to fit in u64 (expected 8 bytes, found {}).", input.len())); // Updated error creation
//	}
//
//	let mut bytes = [0u8; 8];
//	bytes[..input.len()].copy_from_slice(&input);
//	let nonce = u64::from_be_bytes(bytes); 
//
//	Ok(nonce)
//}
//
///// Sets the nonce value in a binary file.
/////
///// This function converts the provided `u64` value to a byte array in big-endian order and writes it to
///// the specified nonce file. If the file does not exist, it will be created.
/////
///// # Parameters
/////
///// - `value`: The `u64` value to set as the nonce.
///// - `file`: An optional path to a specific nonce file.
///// - `home`: An optional base path; the home directory will be used if not provided.
///// - `dont_overwrite`: A flag indicating whether to prevent overwriting an existing nonce file.
/////
///// # Returns
/////
///// - `Ok(())` if the nonce is successfully written to the file.
///// - `Err(eyre::Error)` if an error occurs while retrieving the nonce file path or writing its contents.
//pub fn import(value: u64, file: Option<&Path>, home: Option<&Path>, dont_overwrite: bool) -> Result<()> {
//
//    let nonce_file = get_nonce_file(file, home)
//        .context("Failed to get nonce file path")?;
//
//    // Check if the nonce file already exists and handle the dont_overwrite flag
//    if dont_overwrite && nonce_file.exists() {
//        return Err(eyre::eyre!(
//			"Nonce file already exists at {:?}. Use --dont-overwrite to prevent overwriting.",
//			nonce_file
//		));
//    }
//
//    // Create or open the nonce file in binary write mode
//    let mut file = File::create(&nonce_file)
//        .with_context(|| format!("Failed to create nonce file at {:?}", nonce_file))?;
//
//    // Write the new nonce value as bytes
//    file.write_all(&value.to_be_bytes())
//        .with_context(|| format!("Failed to write to nonce file at {:?}", nonce_file))?;
//
//    Ok(())
//}

///// A Nonce struct that holds both the binary (bytes) and decimal representation of the nonce.
//#[derive(Debug, Clone, PartialEq)]
//pub struct Nonce {
//    /// Binary representation of the nonce.
//    bytes: Vec<u8>,
//
//    /// Decimal representation of the nonce.
//    decimal: u64,
//}
//
//impl Nonce {
//    /// Constructs a new `Nonce` from a decimal value.
//    pub fn from_decimal(decimal: u64) -> Self {
//        // Convert decimal to bytes
//        let bytes = decimal.to_be_bytes().to_vec();
//        Self { bytes, decimal }
//    }
//
//    /// Constructs a new `Nonce` from a binary (bytes) value.
//    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, eyre::Error> {
//        // Convert bytes to decimal
//        let decimal = u64::from_be_bytes(bytes[..].try_into().map_err(|_| eyre::eyre!("Invalid nonce length"))?);
//        Ok(Self { bytes, decimal })
//    }
//
//    /// Returns the decimal representation of the nonce.
//    pub fn decimal(&self) -> u64 {
//        self.decimal
//    }
//
//    /// Returns the binary representation of the nonce.
//    pub fn bytes(&self) -> &[u8] {
//        &self.bytes
//    }
//
//    pub fn from_input(input: Option<&str>, bytes: Option<Vec<u8>>,)  -> Result<Self, eyre::Error>  {
//
//        // If bytes are provided, ignore everything else
//        if let Some(input_bytes) = bytes {
//            return Ok(Self::from_bytes(input_bytes)?);
//        }
//
//        // Check if input is provided
//        if let Some(input_str) = input {
//
//            // Check if the input string can be parsed as a decimal number
//            if let Ok(decimal_value) = input_str.parse::<u64>() {
//                return Ok(Nonce::from_decimal(decimal_value));
//            }
//        }
//
//        // If input was not valid, try to get the nonce from the specified file
//        let nonce_file_path = construct_path(
//            input,
//            Some(&Path::new(".orga-wallet").join("nonce")),
//        )?;
//
//        match binary_or_text_file(&nonce_file_path)? {
//            Data::Binary(bytes) => Nonce::from_bytes(bytes),
//            Data::Text(input_str) => {
//                // Directly parse the string as a decimal
//                let decimal = input_str.parse::<u64>()
//                    .map_err(|_| eyre::eyre!("Invalid decimal string for nonce"))?;
//                Ok(Nonce::from_decimal(decimal))
//            },
//        }
//    }
//
//    pub fn to_output(&self, output: Option<&str>, dont_overwrite: bool) -> Result<(), eyre::Error> {
//        match output {
//            Some(output_str) => {
//                // Construct the path for the output file
//                let nonce_file_path = construct_path(
//                    Some(output_str),
//                    Some(&Path::new(".orga-wallet").join("nonce")),
//                )?;
//
//                // Check if the file exists and dont_overwrite flag is true
//                if dont_overwrite && Path::new(&nonce_file_path).exists() {
//                    return Err(eyre!("File already exists and overwriting is disabled"));
//                }
//
//                // Write the nonce as bytes to the specified file
//                std::fs::write(nonce_file_path, self.bytes())?;
//                Ok(())
//            }
//            None => {
//                // Write raw bytes to stdout for piping to other commands
//                let stdout = io::stdout();
//                let mut handle = stdout.lock();
//                handle.write_all(&self.bytes())?;
//                Ok(())
//            }
//        }
//    }
//
////    /// Display the nonce as a string.
////    pub fn display(&self) -> String {
////        format!("Nonce: Decimal = {}, Bytes = {:?}", self.decimal, self.bytes)
////    }
//}
