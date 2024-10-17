use cosmrs::AccountId;
use cosmrs::crypto::PublicKey;
use cosmrs::crypto::secp256k1::SigningKey;
use eyre::ContextCompat;
use eyre::Result;
use eyre::WrapErr;
use fmt::input::binary_or_text_file;
use fmt::input::Data;
use fmt::input::read_stdin;
use once_cell::sync::OnceCell;
use rand::Rng;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use chrono::{Utc, DateTime};

fn get_file<P: AsRef<Path>>(
    file: Option<P>,
    base_path: Option<P>,
    sub_path: Option<PathBuf>,
) -> Result<PathBuf> {
    match file {
        Some(file_path) => Ok(file_path.as_ref().to_path_buf()),
        None => {
            let base_path = base_path
                .map(|p| p.as_ref().to_path_buf())
                .or_else(dirs::home_dir)
                .wrap_err("Could not determine base path")?;

            let sub_path = sub_path
                .ok_or_else(|| eyre::eyre!("Subpath must be provided"))?;

            Ok(base_path.join(sub_path))
        }
    }
}

/// Retrieves the path to the private key file.
///
/// This function checks if the `file` or `home` paths are provided. If neither is specified,
/// it defaults to `$HOME/.orga-wallet/privkey`. If both are provided, the `file` path takes precedence.
///
/// The private key file is typically stored in the user's home directory at:
///   `$HOME/.orga-wallet/privkey`
///
/// Users can specify an arbitrary file path or allow the function to infer the path from the 
/// provided home directory. The function internally calls `get_file`, which does not specify 
/// the subpath `.orga-wallet/privkey`.
///
/// # Arguments
///
/// * `file` - An optional reference to a specific file path.
/// * `home` - An optional reference to a home directory path.
///
/// # Returns
///
/// Returns a `Result<PathBuf>`, containing the resolved path to the private key file 
/// or an error if the path cannot be determined.
fn get_privkey_file<P: AsRef<Path>>(
    file: Option<P>,
    base_path: Option<P>,
) -> Result<PathBuf> {
    let sub_path = Path::new(".orga-wallet").join("privkey");
    get_file(file, base_path, Some(sub_path))
}

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
    timestamp:   DateTime<Utc>,
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
    pub fn new(
        bytes: [u8; 32],
        timestamp: Option<DateTime<Utc>>,
    ) -> Result<Self> {

        Ok(Self {
            bytes,
            hex:         OnceCell::new(),
            signing_key: OnceCell::new(),
            public_key:  OnceCell::new(),
            account_id:  OnceCell::new(),
            address:     OnceCell::new(),
            timestamp:   timestamp.unwrap_or_else(Utc::now),
        })
    }

    /// Creates a new `Privkey` from a hexadecimal string representation of the private key.
    ///
    /// # Arguments
    ///
    /// * `hex_str` - A string slice containing the hexadecimal representation of the private key.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Self>` containing the new `Privkey` instance or an error if the input
    /// is invalid or cannot be parsed into a valid private key.
    pub fn import(hex_str: &str, timestamp: Option<DateTime<Utc>>) -> Result<Self> {

        // Strip leading and trailing whitespace
        let hex_str = hex_str.trim();

        // Ensure the hex string is the correct length (64 characters for 32 bytes).
        if hex_str.len() != 64 {
            return Err(eyre::eyre!("Hex string must be 64 characters long."));
        }

        // Decode the hex string into bytes.
        let decoded_bytes = hex::decode(hex_str)
            .map_err(|e| eyre::eyre!("Failed to decode hex string: {}", e))?;

        // Ensure the decoded byte length is 32 bytes.
        if decoded_bytes.len() != 32 {
            return Err(eyre::eyre!("Decoded key must be 32 bytes long."));
        }

        // Convert the decoded bytes into a fixed array.
        let mut new_bytes = [0u8; 32];
        new_bytes.copy_from_slice(&decoded_bytes);

        // Use the new constructor to create a PrivKey instance.
        PrivKey::new(new_bytes, timestamp)
    }

    pub fn stdin(max_attempts: usize, timeout: Duration) -> Result<Self> {
        let timestamp = Some(Utc::now());
        // Attempt to read stdin as either binary or text data
        match read_stdin(max_attempts, timeout)? {
            Data::Binary(bytes) => Self::new(
                bytes.try_into()
                    .map_err(|_| eyre::eyre!("Invalid byte array size for new key"))?, 
                timestamp
            ),
            Data::Text(hex_str) => Self::import(&hex_str, timestamp),
        }
    }

    /// Loads an existing key from a file or generates a new one if the file doesn't exist.
    pub fn load<P: AsRef<Path>>(path: P, new: bool) -> Result<Self> {
        let timestamp = Some(Utc::now());
        let path = path.as_ref();

        if path.exists() {
            // Attempt to read the file as either binary or text data
            match binary_or_text_file(&path)? {
                Data::Binary(bytes) => Self::new(
                    bytes.try_into()
                        .map_err(|_| eyre::eyre!("Invalid byte array size for new key"))?,
                    timestamp,
                ),
                Data::Text(hex_str) => Self::import(&hex_str, timestamp),
            }
        } else if new {
            // If the file doesn't exist and `new` is true, generate a new key.
            let mut rng = rand::thread_rng();
            let mut random_bytes = [0u8; 32];
            rng.fill(&mut random_bytes);

            println!("Warning: A new key has been generated. Please back it up securely.");

            // Save the generated key to the file.
            let mut file = std::fs::File::create(path)
                .wrap_err_with(|| format!("Failed to create key file: {:?}", path))?;
            file.write_all(&random_bytes)
                .wrap_err_with(|| format!("Failed to write new key to file: {:?}", path))?;

            PrivKey::new(random_bytes, timestamp)
        } else {
            // If the file doesn't exist and `new` is false, return an error.
            Err(eyre::eyre!(
                "PrivKey file does not exist, and the `new` option was not set to true."
            ))
        }
    }

    /// Creates a new `Privkey` from a file path, checking the home directory if needed.
    ///
    /// # Arguments
    ///
    /// * `file` - An optional reference to a `Path` representing the file containing the private key.
    /// * `home` - An optional reference to a `Path` representing the home directory to check for the file.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Self>` containing the new `Privkey` instance or an error if the file 
    /// cannot be found or read.
    pub fn load_from_file_or_home(file: Option<&Path>, home: Option<&Path>, new: bool) -> Result<Self> {
        Self::load(get_privkey_file(file, home)?, new)
    }

    /// Retrieves the hex-encoded private key.
    ///
    /// This method ensures that the hex representation is computed lazily and only when needed.
    ///
    /// # Returns
    ///
    /// Returns a `Result<&str, eyre::Report>` containing the hex-encoded private key or an
    /// error if the computation fails.
    pub fn export(&self) -> Result<&str> {
        self.hex.get_or_try_init(|| {
            Ok(hex::encode(self.bytes))
        }).map(|hex_str| hex_str.as_str())
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

    pub fn save_to_file_or_home<P: AsRef<Path>>(&self,
        file: Option<P>,
        home: Option<P>,
        force: bool
    ) -> Result<()> {
        self.save(get_privkey_file(file, home)?, force)
    }
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
        Self::new(self.bytes.clone(), Some(self.timestamp))
            .expect("Failed to create Privkey from cloned bytes")
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

impl FromBytes for [u8; 32] {
    fn privkey(self) -> Result<PrivKey> {
        PrivKey::new(self, Some(Utc::now()))
    }
}

// Implementing FromBytes for Vec<u8>
impl FromBytes for Vec<u8> {
    fn privkey(self) -> Result<PrivKey> {
        // Ensure the length is 32 bytes before creating a PrivKey
        if self.len() != 32 {
            return Err(eyre::eyre!("Vec<u8> must be 32 bytes long."));
        }

        // Convert Vec<u8> into an array of 32 bytes
        let mut array = [0u8; 32];
        array.copy_from_slice(&self);

        PrivKey::new(array, Some(Utc::now()))
    }
}

impl FromHex for &str {
    fn privkey(self) -> Result<PrivKey> {
        PrivKey::import(self, Some(Utc::now()))
    }
}

impl FromHex for String {
    fn privkey(self) -> Result<PrivKey> {
        PrivKey::import(&self, Some(Utc::now()))
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

