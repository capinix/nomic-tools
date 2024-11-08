use cosmrs::AccountId;
use cosmrs::crypto::PublicKey;
use cosmrs::crypto::secp256k1::SigningKey;
//use eyre::ContextCompat;
use eyre::eyre;
use eyre::Result;
use eyre::WrapErr;
use crate::functions::read_stdin;
use log::info;
use once_cell::sync::OnceCell;
use rand::Rng;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;


///// Retrieves the private key based on the provided input, relative path, or bytes.
/////
///// This function checks various input methods for retrieving a private key:
///// - If `bytes` are provided, it directly converts them into a `PrivKey`.
///// - If an `input` string is provided, it first checks if it's a valid hex string of 64 characters.
///// - If the hex string decodes to exactly 32 bytes, it converts them to a `PrivKey`.
///// - If the input is not a valid hex string, it attempts to resolve the path using `construct_path`
/////   to get the private key from a file or profile.
/////
///// # Arguments
/////
///// * `input` - An optional string slice that can be a hex string, file path, or profile name.
///// * `sub_path` - An optional reference to a `Path` used to extend the input path.
///// * `bytes` - An optional array of bytes that can be directly converted to a `PrivKey`.
/////
///// # Returns
/////
///// * `Result<PrivKey>` - Returns the retrieved private key if successful, or an error if any checks fail.
/////
///// # Errors
/////
///// This function returns an error if:
///// - The provided hex string cannot be decoded into 32 bytes.
///// - The `construct_path` call fails to resolve the path.
/////
///// # Example
/////
///// ```
///// let privkey = get_privkey(Some("your_hex_string_here"), None, None);
///// ```
//pub fn get_privkey<S: AsRef<str>>(
//    input: Option<S>,
//    sub_path: Option<S>,
//    bytes: Option<[u8; 32]>,
//) -> Result<PrivKey> {
//
//    // If bytes are provided, ignore everything else
//    if let Some(input_bytes) = bytes {
//        // use the FromBytes Trait
//        return input_bytes.privkey();
//    }
//
//    //let sub_path = sub_path.map(|p| {
//    //    let sub_path_path: &Path = p.as_ref();
//    //    sub_path_path.to_path_buf()
//    //});
//
//
//    // Check if input is provided
//    if let Some(input_str) = &input {
//
//        // Step 1: Check if the input string is a valid hex string (64 characters)
//        let input_string: &str = input_str.as_ref();
//        if input_string.len() == 64 {
//            // Attempt to decode the hex string
//            if let Ok(decoded_bytes) = hex::decode(input_string) {
//                if decoded_bytes.len() == 32 {
//                    let mut array = [0u8; 32];
//                    array.copy_from_slice(&decoded_bytes);
//                    return array.privkey();
//                } else {
//                    return Err(eyre!("Hex string decoded but not 32 bytes: {:?}", decoded_bytes));
//                }
//            }
//            // If decoding fails, continue to the next checks
//        } 
//        // If length is not 64, continue to the next checks
//    }
//
//    // use the FromPath Trait to get the key
//    construct_path(input, sub_path)?.privkey()
//}

//fn get_file<P: AsRef<Path>>(
//    file: Option<P>,
//    base_path: Option<P>,
//    sub_path: Option<PathBuf>,
//) -> Result<PathBuf> {
//    match file {
//        Some(file_path) => {
//                let file: &Path = file_path.as_ref();
//                Ok(file.to_path_buf())
//            },
//        None => {
//            let base_path = base_path
//                .map(|p| {
//                    let base_path: &Path = p.as_ref();
//                    base_path.to_path_buf()
//                })
//                .or_else(dirs::home_dir)
//                .wrap_err("Could not determine base path")?;
//
//            let sub_path = sub_path
//                .map(|p| {
//                    let sub_path: &Path = p.as_ref();
//                    sub_path.to_path_buf()
//                })
//                .ok_or_else(|| eyre::eyre!("Subpath must be provided"))?;
//
//            Ok(base_path.join(sub_path))
//        }
//    }
//}

///// Retrieves the path to the private key file.
/////
///// This function checks if the `file` or `home` paths are provided. If neither is specified,
///// it defaults to `$HOME/.orga-wallet/privkey`. If both are provided, the `file` path takes precedence.
/////
///// The private key file is typically stored in the user's home directory at:
/////   `$HOME/.orga-wallet/privkey`
/////
///// Users can specify an arbitrary file path or allow the function to infer the path from the 
///// provided home directory. The function internally calls `get_file`, which does not specify 
///// the subpath `.orga-wallet/privkey`.
/////
///// # Arguments
/////
///// * `file` - An optional reference to a specific file path.
///// * `home` - An optional reference to a home directory path.
/////
///// # Returns
/////
///// Returns a `Result<PathBuf>`, containing the resolved path to the private key file 
///// or an error if the path cannot be determined.
//fn get_privkey_file<P: AsRef<Path>>(
//    file: Option<P>,
//    base_path: Option<P>,
//) -> Result<PathBuf> {
//    let sub_path = Path::new(".orga-wallet").join("privkey");
//    // Passing `sub_path.as_path()` to match the expected type for `get_file`
//    get_file(file, base_path, Some(sub_path))
//
//}

/// Represents a private key.
///
/// This struct encapsulates a 32-byte private key along with its hex representation, signing key, 
/// public key, account ID, and address. The fields are lazily initialized to improve performance 
/// and resource usage.
///
/// # Fields
///
/// * `bytes`: 32-byte representation of the private key.
/// * `hex`: Hexadecimal representation of the private key as a string.
/// * `signing_key`: cosmrs Signing key derived from the private key.
/// * `secret_key` : orga Signing key derived from the private key.
/// * `public_key` : Public key derived from the signing key.
/// * `account_id` : Account ID associated with the public key.
/// * `address`: String representation of the account ID.
pub struct PrivKey {
    /// 32-byte representation of the private key.
    bytes: [u8; 32],
    /// Hexadecimal representation of the private key as a string.
    hex:         OnceCell<String>,
    /// cosmrs Signing key derived from the private key.
    signing_key: OnceCell<SigningKey>,
    /// cosmrs Publickey derived from the signing key.
    public_key:  OnceCell<PublicKey>,
    /// cosmrs AccountId associated with the public key.
    account_id:  OnceCell<AccountId>,
    /// String representation of the AccountId.
    address:     OnceCell<String>,
}

impl PrivKey {
    /// Creates a new `PrivKey` from the provided byte array.
    ///
    /// # Arguments
    ///
    /// * `bytes` - A array containing the private key bytes. Must be exactly 32 bytes in length.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Self>` containing the new `Privkey` instance or an error if the byte 
    /// length is invalid or if the bytes do not represent a valid private key.
    /// If `bytes` is `None`, generates a new random 32-byte key.
    pub fn new(bytes: Option<[u8; 32]>) -> Self {
        let bytes = match bytes {
            Some(b) => b,
            None => {
                println!("Warning: A new key has been generated. Please back it up securely.");
                let mut rng = rand::thread_rng();
                let mut random_bytes = [0u8; 32];
                rng.fill(&mut random_bytes); // This is guaranteed to succeed
                random_bytes // Return the generated random bytes
            },
        };

        Self {
            bytes,
            hex: OnceCell::new(),
            signing_key: OnceCell::new(),
            public_key: OnceCell::new(),
            account_id: OnceCell::new(),
            address: OnceCell::new(),
        }
    }

    /// Attempts to interpret the input data as a file path, a 32-byte binary
    ///   key, or a 64-character hexadecimal string.
    ///
    /// This function follows these steps:
    /// 1. First, it checks if the input data can be interpreted as a UTF-8
    ///    string and a valid file path.
    ///    - If the file exists, it reads the file's contents and recursively
    ///      applies this function to the file data.
    /// 2. If the input is not a valid path or does not exist, it checks if the 
    ///    data length is exactly 32 bytes and, if so, treats it as binary key data.
    /// 3. If the data length is not 32 bytes, it then interprets the input as a
    ///    potential 64-character hexadecimal string.
    ///
    /// # Parameters
    ///
    /// - `data`: A generic input of type `T`, where `T` implements `AsRef<Vec<u8>>`.
    ///    This can represent binary data, a hexadecimal string, or a file path with key content.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Self>`, with:
    /// - `Ok(Key::Binary([u8; 32]))` if the input is valid 32-byte binary data.
    /// - `Ok(Key::Binary([u8; 32]))` if the input is a valid 64-character hexadecimal string that converts to a 32-byte binary key.
    /// - `Ok(Self::new(None))` if the input refers to an empty file.
    /// - An error if none of these formats match or if file reading encounters an issue.
    ///
    /// # Errors
    ///
    /// This function may return an error in the following cases:
    /// - If the input data is empty.
    /// - If the input data does not represent a valid 32-byte binary key or 64-character hexadecimal string.
    /// - If the file path exists but the file cannot be read (e.g., permissions issue, file not found).
    /// - If the hexadecimal string cannot be converted to a 32-byte array due to incorrect length or invalid characters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example for a 64-character hexadecimal string
    /// let key_from_hex = Key::from_vec("aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899".as_bytes().to_vec())?;
    /// 
    /// // Example for 32-byte binary data
    /// let key_from_binary = Key::from_vec(vec![0u8; 32])?;
    /// 
    /// // Example for a file path
    /// let key_from_file = Key::from_vec("path/to/keyfile".as_bytes().to_vec())?;
    /// ```
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
                        return Ok(Self::new(None));
                    }
                },
                Err(_) => {
                    // If the file cannot be read, return an error
                    return Err(eyre::eyre!("Cannot read file at path: {:?}", path));
                }
            }
        } else {
            info!("Input string is not a valid file path; checking for binary or hex...");
        }

        // Check if the data length is exactly 32 bytes; if so, treat it as binary.
        if buf.len() == 32 {
            let mut binary_data = [0u8; 32];
            binary_data.copy_from_slice(buf);
            return Ok(Self::new(Some(binary_data)));
        } else {
            info!("Not a valid 32-byte key; performing further checks...");
        }

        // Otherwise, attempt to interpret the bytes as a UTF-8 string.
        if let Ok(text_data) = String::from_utf8(buf.to_vec()) {
            // Check if it's a valid 64-character hex string.
            if text_data.len() == 64 && text_data.chars().all(|c| c.is_digit(16)) {
                // Decode the hex string into a vector of bytes
                if let Ok(decoded_bytes) = hex::decode(&text_data) {
                    // Convert the decoded vector to [u8; 32] if it's exactly 32 bytes
                    if decoded_bytes.len() == 32 {
                        let mut binary_data = [0u8; 32];
                        binary_data.copy_from_slice(&decoded_bytes);
                        return Ok(Self::new(Some(binary_data)));
                    } else {
                        return Err(eyre::eyre!("Hex string is not the correct length for [u8; 32]"));
                    }
                } else {
                    return Err(eyre::eyre!("Failed to decode hex string to binary"));
                }
            } else {
                return Err(eyre::eyre!("Data is not hex;..."));
            }
        } else {
            return Err(eyre::eyre!("Input data is not valid UTF-8."));
        }
    }

    /// Imports a private key from a hex string or file path.
    ///
    /// This function accepts input as either a 64-character hex string or a file path containing
    /// the key data. If given a file path, the function reads the file contents, which can be in 
    /// hex or binary format, and imports it as a private key.
    ///
    /// # Parameters
    /// - `data`: A generic input type that implements `AsRef<str>`, which may be:
    ///   - A hex string directly representing the key
    ///   - A file path where the key is stored, containing either hex or binary data
    ///
    /// # Returns
    /// - `Result<Self>`:
    ///   - On success, returns an instance of the type containing the imported key.
    ///   - On failure, returns an error explaining why the import failed, such as hex decoding errors
    ///     or an invalid key format.
    ///
    /// # Errors
    /// This function may return an error if:
    /// - The hex string cannot be decoded to binary data.
    /// - The resulting binary data does not match the expected 32-byte length.
    /// - The file path does not exist or cannot be read.
    ///
    /// # Example
    /// ```
    /// let key = import("your_hex_or_file_path")?;
    /// ```
    pub fn import<S: AsRef<str>>(input: S) -> Result<Self> {
        let input_str: &str = input.as_ref();
        let input_vec = input_str.as_bytes().to_vec();
        Self::from_vec(input_vec)
    }

    /// Loads a private key from a specified file path or generates and saves a new one if requested.
    ///
    /// This function tries to load an existing key from the specified file path. If the file
    /// does not exist and the `new` flag is set to `true`, it generates a new private key,
    /// saves it to the specified file path, and returns the new key. If `new` is `false` and
    /// the file does not exist, an error is returned.
    ///
    /// # Parameters
    /// - `path`: A reference to a path where the key file is located or will be saved if a new key is created.
    ///   The path type implements `AsRef<Path>`.
    /// - `new`: A boolean flag that determines behavior when the file does not exist:
    ///   - If `true`, generates a new key and saves it if the file cannot be imported.
    ///   - If `false`, returns an error if the file is missing.
    ///
    /// # Returns
    /// - `Result<Self>`:
    ///   - On success, returns an instance containing the loaded or newly created key.
    ///   - On failure, returns an error if the file could not be read, if the key import failed,
    ///     or if the file does not exist and `new` is `false`.
    ///
    /// # Errors
    /// - Returns an error if the file does not exist and `new` is `false`.
    /// - Returns an error if the file exists but does not contain a valid key format.
    /// - Returns an error if the newly generated key could not be saved.
    ///
    /// # Example
    /// ```
    /// let key = KeyType::load("/path/to/keyfile", true)?;
    /// ```
    pub fn load<P: AsRef<Path>>(path: P, new: bool) -> Result<Self> {

        let path: &Path = path.as_ref();
        let path_str = path.to_string_lossy();

        // Attempt to import the key from the file
        if let Ok(key_instance) = Self::import(path_str) {
            return Ok(key_instance);
        }

        // If the import fails and a new key should be generated
        if new {
            // Create a new key
            let privkey = Self::new(None);
            // Save the new key to the specified path
            privkey.save(path, true)?;

            return Ok(privkey); // Return the newly created key
        }

        // If the file does not exist and 'new' is false, return an error
        Err(eyre!("Key file does not exist and 'new' is set to false"))
    }

    /// Reads a private key from standard input (stdin).
    /// 
    /// This function will read data from stdin for a specified number of attempts and timeout duration.
    /// It interprets the input as either:
    /// - A 64-character hexadecimal string (converted to binary data)
    /// - A 32-byte binary array (processed directly)
    ///
    /// # Parameters
    ///
    /// - `max_attempts`: The maximum number of attempts to read from stdin before failing.
    /// - `timeout_in_seconds`: The timeout duration in seconds for each read attempt.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Self>`, which will be:
    /// - `Ok(Self)`: The instance if the input is valid and successfully parsed.
    /// - An error if the input cannot be parsed as a key due to format issues or reading errors.
    ///
    /// # Errors
    ///
    /// This function may return an error if:
    /// - The input cannot be interpreted as a valid 64-character hex string or a 32-byte binary array.
    /// - Conversion from hex to binary fails.
    /// - The hex string does not convert to exactly 32 bytes.
    /// - Reading from stdin encounters issues (e.g., timeouts).
    ///
    /// # Example
    ///
    /// ```rust
    /// let key_instance = Privkey::stdin(3, 5).expect("Failed to read key from stdin");
    /// ```
    pub fn stdin(max_attempts: usize, timeout_in_seconds: u64) -> Result<Self> {
        let data_bytes = read_stdin(max_attempts, timeout_in_seconds)?;
        Self::from_vec(data_bytes)
    }

//    /// Creates a new `Privkey` from a file path, checking the home directory if needed.
//    ///
//    /// # Arguments
//    ///
//    /// * `file` - An optional reference to a `Path` representing the file containing the private key.
//    /// * `home` - An optional reference to a `Path` representing the home directory to check for the file.
//    ///
//    /// # Returns
//    ///
//    /// Returns a `Result<Self>` containing the new `Privkey` instance or an error if the file 
//    /// cannot be found or read.
//    #[allow(dead_code)]
//    pub fn load_from_file_or_home(file: Option<&Path>, home: Option<&Path>, new: bool) -> Result<Self> {
//        Self::load(get_privkey_file(file, home)?, new)
//    }

    pub fn bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Retrieves the hex-encoded private key.
    ///
    /// This method ensures that the hex representation is computed lazily and only when needed.
    ///
    /// # Returns
    ///
    /// Returns a `Result<&str, eyre::Report>` containing the hex-encoded private key or an
    /// error if the computation fails.
    pub fn export(&self) -> &str {
        self.hex.get_or_init(|| hex::encode(self.bytes)).as_str()
    }

    /// Lazily retrieves or computes the signing key from the private key bytes.
    ///
    /// # Returns
    ///
    /// Returns a `Result<&SigningKey, eyre::Report>`, providing a reference to the signing key 
    /// or an error if the signing key creation fails.
    pub fn signing_key(&self) -> Result<&SigningKey> {
    // Get the signing key, initializing if necessary
        self.signing_key.get_or_try_init(|| {
            SigningKey::from_slice(&self.bytes)
                .map_err(|e| eyre::eyre!("Invalid secret key: {}", e))
        })
    }

    /// Lazily retrieves or computes the public key from the signing key.
    ///
    /// # Returns
    ///
    /// Returns a `Result<&PublicKey, eyre::Report>`, providing a reference to the public key 
    /// or an error if the public key computation fails.
    pub fn public_key(&self) -> Result<&PublicKey> {
        self.public_key.get_or_try_init(|| {
            Ok(self.signing_key()?.public_key())
        })
    }

    /// Lazily retrieves or computes the account ID from the public key.
    ///
    /// # Returns
    ///
    /// Returns a `Result<&AccountId, eyre::Report>`, providing a reference to the account ID 
    /// or an error if the account ID computation fails.
    pub fn account_id(&self) -> Result<&AccountId> {
        self.account_id.get_or_try_init(|| {
            self.public_key()?.account_id("nomic")
                .map_err(|e| eyre::eyre!("Failed to get address from public key: {}", e))
        })
    }

    /// Lazily retrieves or computes the address associated with the account ID.
    ///
    /// # Returns
    ///
    /// Returns a `Result<String, eyre::Report>` containing the address as a string or an
    /// error if the address computation fails.
    pub fn address(&self) -> Result<&str> {
        self.address.get_or_try_init(|| {
            Ok(self.account_id()?.to_string())
        }).map(|s| s.as_str())
    }

    /// Saves the key to a binary file at the given path, with a `force` flag to control overwriting.
    pub fn save<P: AsRef<Path>>(&self,
        file_path: P,
        force: bool,
    ) -> Result<()> {
        let path = file_path.as_ref();

        if path.exists() {
            // If the file exists and `force` is true, overwrite the file
            if force {
                let mut file = File::create(path)
                    .wrap_err_with(|| format!("Failed to overwrite key file: {:?}", path))?;
                file.write_all(&self.bytes)
                    .wrap_err_with(|| format!("Failed to write key to file: {:?}", path))?;
                println!("PrivKey successfully overwritten to {:?}", path);
                Ok(())
            } else {
                // If `force` is not true, return an error
                Err(eyre::eyre!("File already exists at {:?} and `force` is not enabled.", path))
            }
        } else {
            // If the file does not exist, create a new file and write the key bytes
            let mut file = File::create(path)
                .wrap_err_with(|| format!("Failed to create key file: {:?}", path))?;
            file.write_all(&self.bytes)
                .wrap_err_with(|| format!("Failed to write key to new file: {:?}", path))?;
            println!("PrivKey successfully saved to {:?}", path);
            Ok(())
        }
    }

//    pub fn save_to_file_or_home<P: AsRef<Path>>(&self,
//        file: Option<P>,
//        home: Option<P>,
//        force: bool
//    ) -> Result<()> {
//        self.save(get_privkey_file(file, home)?, force)
//    }
}

// Custom Debug implementation for Profile
impl std::fmt::Debug for PrivKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Handle the Result manually, converting it into a debug-friendly format
        let address_str = match self.address() {
            Ok(address) => format!("{:?}", address),     // Use the address value on success
            Err(_) => "Error fetching address".to_string(), // Handle error case
        };
        write!(
            f,
            "PrivKey {{ address: {} }}",
            address_str
        )
    }
}

impl PartialEq for PrivKey {
    fn eq(&self, other: &Self) -> bool {
        self.bytes.eq_ignore_ascii_case(&other.bytes)
    }
}

/// Implementing the `Clone` trait for `Privkey`.
///
/// This implementation provides a way to clone a `Privkey` instance by only cloning its
/// byte representation. The `SigningKey` and other fields that are lazily initialized cannot
/// be cloned directly due to the lack of a `Clone` implementation for the `SigningKey` type.
///
/// As a result, this implementation utilizes the `new` method, which reinitializes the necessary 
/// components using the cloned byte vector. This way, the `Privkey` can still be duplicated 
/// while adhering to the constraints of the underlying cryptographic types.
impl Clone for PrivKey {
    fn clone(&self) -> Self {
        Self::new(Some(self.bytes.clone()))
    }
}

/// A trait for converting a byte vector into a `PrivKey`.
///
/// This trait provides a consistent interface for creating `Privkey` instances
/// from byte representations across different types.
#[allow(dead_code)]
pub trait FromBytes {
    /// Converts the implementing type into a `PrivKey`.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `PrivKey`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// use your_crate::Privkey; // Replace with the actual path to your Privkey type
    /// use your_crate::FromBytes; // Replace with the actual path to your FromBytes trait
    ///
    /// // Create a byte vector representing a private key
    /// let private_key_bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04];
    ///
    /// // Convert the byte vector to a PrivKey
    /// let Key = private_key_bytes.PrivKey().expect("Failed to create Privkey from bytes");
    ///
    /// // Use the Privkey as needed
    /// ```
    fn privkey(self) -> Result<PrivKey>;
}

impl FromBytes for [u8; 32] {
    fn privkey(self) -> Result<PrivKey> {
        Ok(PrivKey::new(Some(self)))
    }
}

// Implementing FromBytes for Vec<u8>
impl FromBytes for Vec<u8> {
    fn privkey(self) -> Result<PrivKey> {
        PrivKey::from_vec(self)
    }
}

/// A trait for converting a hexadecimal string into a `Privkey`.
///
/// This trait provides a consistent interface for creating `Privkey` instances 
/// from hexadecimal representations across different types.
pub trait FromHex {
    /// Converts the implementing type into a `PrivKey`.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `PrivKey`.
    ///
    /// # Examples
    ///
    /// Basic usage with a string literal:
    /// ```
    /// use your_crate::Privkey; // Replace with the actual path to your Privkey type
    /// use your_crate::FromHex; // Replace with the actual path to your FromHex trait
    ///
    /// // A hexadecimal string representing a private key
    /// let hex_str: &str = "0102030405060708090a0b0c0d0e0f10";
    ///
    /// // Convert the hexadecimal string to a Privkey
    /// let privkey = hex_str.privkey().expect("Failed to create Privkey from hex string");
    ///
    /// // Use the Privkey as needed
    /// ```
    #[allow(dead_code)]
    fn privkey(self) -> Result<PrivKey>;
}

/// A trait for converting a file path into a `PrivKey`.
///
/// This trait provides a consistent interface for creating `Privkey` instances 
/// from file paths across different types.
pub trait FromPath {
    /// Converts the implementing type into a `PrivKey`.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `PrivKey`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// use your_crate::Privkey; // Replace with the actual path to your Privkey type
    /// use your_crate::FromPath; // Replace with the actual path to your FromPath trait
    ///
    /// // A file path representing a private key
    /// let file_path = Path::new("path/to/private_key_file");
    ///
    /// // Convert the file path to a Privkey
    /// let privkey = file_path.privkey().expect("Failed to create Privkey from file");
    ///
    /// // Use the Privkey as needed
    /// ```
    fn privkey(self) -> Result<PrivKey>;
}

impl FromHex for &str {
    fn privkey(self) -> Result<PrivKey> {
        PrivKey::import(self)
    }
}

impl FromHex for String {
    fn privkey(self) -> Result<PrivKey> {
        PrivKey::import(&self)
    }
}

impl FromPath for &Path {
    fn privkey(self) -> Result<PrivKey> {
        PrivKey::load(self, true)
    }
}

impl FromPath for PathBuf {
    fn privkey(self) -> Result<PrivKey> {
        PrivKey::load(&self, true)
    }
}

