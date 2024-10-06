use cosmrs::{AccountId, crypto::{secp256k1::SigningKey, PublicKey}};
use crate::functions::get_file;
use eyre::{Result, WrapErr};
use fmt::input::{Data, binary_or_text_file, read_stdin};
use once_cell::sync::OnceCell;
use std::{fs::File, io::Write, path::{Path, PathBuf}, time::Duration};

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
fn get_privkey_file(file: Option<&Path>, home: Option<&Path>) -> Result<PathBuf> {
    let sub_path = Path::new(".orga-wallet").join("privkey");
    get_file(file, home, Some(&sub_path))
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
/// * `signing_key`: Signing key derived from the private key.
/// * `public_key`: Public key derived from the signing key.
/// * `account_id`: Account ID associated with the public key.
/// * `address`: String representation of the account ID.
pub struct Privkey {
	/// 32-byte representation of the private key.
	bytes: Vec<u8>,
	/// Hexadecimal representation of the private key as a string.
	hex: OnceCell<String>,
	/// Signing key derived from the private key.
	signing_key: OnceCell<SigningKey>,
	/// Public key derived from the signing key.
	public_key: OnceCell<PublicKey>,
	/// Account ID associated with the public key.
	account_id: OnceCell<AccountId>,
	/// String representation of the account ID.
	address: OnceCell<String>,
}

impl Privkey {
    /// Creates a new `Privkey` from the provided byte vector.
    ///
    /// # Arguments
    ///
    /// * `bytes` - A vector containing the private key bytes. Must be exactly 32 bytes in length.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Self>` containing the new `Privkey` instance or an error if the byte 
    /// length is invalid or if the bytes do not represent a valid private key.
	pub fn new_from_bytes(bytes: Vec<u8>) -> Result<Self> {
		if bytes.len() != 32 {
			return Err(eyre::eyre!("Invalid byte length: Must be 32 bytes."));
		}
		Ok(Self {
			bytes,
			hex: OnceCell::new(),
			signing_key: OnceCell::new(),
			public_key: OnceCell::new(),
			account_id: OnceCell::new(),
			address: OnceCell::new(),
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
    pub fn new_from_hex(hex_str: &str) -> Result<Self> {
        // Strip leading and trailing whitespace
        let hex_str = hex_str.trim();

        // Convert the hex string into bytes
        let bytes = hex::decode(hex_str).map_err(|e| {
            eyre::eyre!("Failed to decode hex: {}", e)
        })?;

        // Use the existing `new` method to create the Privkey
        Self::new_from_bytes(bytes)
    }

    /// Creates a new `Privkey` from a file containing either binary or hexadecimal representation.
    ///
    /// # Arguments
    ///
    /// * `file` - A reference to a `Path` representing the file containing the private key.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Self>` containing the new `Privkey` instance or an error if the file 
    /// cannot be read or does not contain valid data.
	pub fn new_from_file(file: &Path) -> Result<Self> {
		// Attempt to read the file as either binary or text data
		match binary_or_text_file(file)? {
			Data::Binary(bytes) => Self::new_from_bytes(bytes),
			Data::Text(hex_str) => Self::new_from_hex(&hex_str),
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
	pub fn new_from_file_or_home(file: Option<&Path>, home: Option<&Path>) -> Result<Self> {
		let path = get_privkey_file(file, home)?;
		Self::new_from_file(&path)
	}

	pub fn new_from_stdin(max_attempts: usize, timeout: Duration) -> Result<Self> {
		// Attempt to read stdin as either binary or text data
		match read_stdin(max_attempts, timeout)? {
			Data::Binary(bytes) => Self::new_from_bytes(bytes),
			Data::Text(hex_str) => Self::new_from_hex(&hex_str),
		}
	}

	/// Retrieves the hex-encoded private key.
	///
	/// This method ensures that the hex representation is computed lazily and only when needed.
	///
	/// # Returns
	///
	/// Returns a `Result<&str, eyre::Report>` containing the hex-encoded private key or an
	/// error if the computation fails.
    pub fn hex(&self) -> String {
		self.hex.get_or_init(|| hex::encode(&self.bytes)).to_string()
    }

	/// Lazily retrieves or computes the signing key from the private key bytes.
	///
	/// # Returns
	///
	/// Returns a `Result<&SigningKey, eyre::Report>`, providing a reference to the signing key 
	/// or an error if the signing key creation fails.
	pub fn signing_key(&self) -> &SigningKey {
		// Get the signing key, initializing if necessary
		self.signing_key.get_or_init(|| {
			SigningKey::from_slice(&self.bytes)
				.expect("Failed to create SigningKey from bytes") // This will panic if the key cannot be created
		})
	}

	/// Lazily retrieves or computes the public key from the signing key.
	///
	/// # Returns
	///
	/// Returns a `Result<&PublicKey, eyre::Report>`, providing a reference to the public key 
	/// or an error if the public key computation fails.
	pub fn public_key(&self) -> &PublicKey {
		self.public_key.get_or_init(|| {
			self.signing_key().public_key()
		})
	}

	/// Lazily retrieves or computes the account ID from the public key.
	///
	/// # Returns
	///
	/// Returns a `Result<&AccountId, eyre::Report>`, providing a reference to the account ID 
	/// or an error if the account ID computation fails.
	pub fn account_id(&self) -> &AccountId {
		self.account_id.get_or_init(|| {
			self.public_key().account_id("nomic")
				.expect("Failed to get address from public key")
		})
	}

	/// Lazily retrieves or computes the address associated with the account ID.
	///
	/// # Returns
	///
	/// Returns a `Result<Cow<str>, eyre::Report>` containing the address as a string or an 
	/// error if the address computation fails.
	pub fn address(&self) -> &str {
		self.address.get_or_init(|| {
			self.account_id().to_string()
		})
	}
}

impl std::fmt::Debug for Privkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Privkey {{ address: {} }}", self.address())
    }
}

impl PartialEq for Privkey {
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
impl Clone for Privkey {
    fn clone(&self) -> Self {
        Self::new_from_bytes(self.bytes.clone()).expect("Failed to create Privkey from cloned bytes")
    }
}

/// A trait for converting a byte vector into a `Privkey`.
///
/// This trait provides a consistent interface for creating `Privkey` instances
/// from byte representations across different types.
#[allow(dead_code)]
pub trait FromBytes {
    /// Converts the implementing type into a `Privkey`.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `Privkey`.
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
    /// // Convert the byte vector to a Privkey
    /// let privkey = private_key_bytes.privkey().expect("Failed to create Privkey from bytes");
    ///
    /// // Use the Privkey as needed
    /// ```
    fn privkey(self) -> Result<Privkey>;
}

/// A trait for converting a hexadecimal string into a `Privkey`.
///
/// This trait provides a consistent interface for creating `Privkey` instances 
/// from hexadecimal representations across different types.
pub trait FromHex {
    /// Converts the implementing type into a `Privkey`.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `Privkey`.
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
    fn privkey(self) -> Result<Privkey>;
}

/// A trait for converting a file path into a `Privkey`.
///
/// This trait provides a consistent interface for creating `Privkey` instances 
/// from file paths across different types.
pub trait FromPath {
    /// Converts the implementing type into a `Privkey`.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `Privkey`.
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
    fn privkey(self) -> Result<Privkey>;
}

impl FromBytes for Vec<u8> {
    fn privkey(self) -> Result<Privkey> {
        Privkey::new_from_bytes(self)
    }
}

impl FromHex for &str {
    fn privkey(self) -> Result<Privkey> {
        Privkey::new_from_hex(self)
    }
}

impl FromHex for String {
    fn privkey(self) -> Result<Privkey> {
        Privkey::new_from_hex(&self)
    }
}

impl FromPath for &Path {
    fn privkey(self) -> Result<Privkey> {
        Privkey::new_from_file(self)
    }
}

impl FromPath for PathBuf {
    fn privkey(self) -> Result<Privkey> {
        Privkey::new_from_file(&self)
    }
}

impl Privkey {
	/// Saves the private key to the specified file or home directory.
	///
	/// This method ensures that either a `file` or `home` path is provided, but not both.
	/// If the file already exists and `force` is `false`, it returns an error to prevent 
	/// overwriting. If `force` is `true`, it will overwrite the existing file.
	///
	/// # Arguments
	///
	/// * `file` - An optional reference to a specific file path for saving the private key.
	/// * `home` - An optional reference to a home directory path to derive the save path.
	/// * `force` - A boolean indicating whether to overwrite an existing file.
	///
	/// # Returns
	///
	/// Returns a `Result<()>`, indicating success or failure of the save operation.
	pub fn save(&self, file: Option<&Path>, home: Option<&Path>, force: bool) -> Result<()> {
		// Check if neither file nor home is provided
		if file.is_none() && home.is_none() {
			return Err(eyre::eyre!("Must specify either a file or a home directory"));
		}

		// Check if both file and home are provided
		if file.is_some() && home.is_some() {
			return Err(eyre::eyre!("Cannot specify both a file and a home directory"));
		}

		// Get the privkey file path
		let output_file = get_privkey_file(file, home)
			.context("Failed to get privkey file path")?;

		// Check if the file exists
		if output_file.exists() {
			// File exists, check if overwrite is allowed
			if force {
				println!("Overwriting existing file: {:?}", output_file);
			} else {
				return Err(eyre::eyre!("File already exists. Use --force to overwrite."));
			}
		} else {
			// File does not exist, proceed to create it
			println!("Creating new file: {:?}", output_file);
		}

		// Write the decoded data to the specified file
		let mut file = File::create(&output_file)
			.context("Failed to create or open the privkey file")?;

		file.write_all(&self.bytes)
			.context("Failed to write to the privkey file")?;

		println!("Private key set successfully.");
		Ok(())
	}
}
