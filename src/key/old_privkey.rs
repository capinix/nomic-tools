use cosmrs::crypto::{secp256k1::SigningKey, PublicKey};
use cosmrs::AccountId;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use eyre::{Result, WrapErr};
use crate::key::{
	get_privkey_file,
};

/// Represents a private key (in byte or hex format) and provides access to the associated 
/// `AccountID`, `SigningKey`, and `PublicKey` using the `cosmrs` crate.
/// 
/// This struct handles private keys that are stored as either a byte array or a hexadecimal string. 
/// The key file often contains a raw byte array, which may include non-printable characters 
/// when read as text. By reading the file as a `Vec<u8>` and performing hex decoding, 
/// the correct hexadecimal representation of the private key is derived.
/// 
/// Upon receiving either a byte array or a hex string, the struct automatically determines 
/// and stores the other representation. With the private key in hand (whether as bytes or hex), 
/// we utilize `cosmrs` to derive the corresponding `SigningKey`, `PublicKey`, and `AccountID` 
/// (which represents the address associated with the key).
///
/// # Fields
/// 
/// - `bytes`: The raw byte stream of the private key, which is hex-encoded binary data.
/// - `hex`: The private key in its hexadecimal string representation.
/// - `account_id`: The `AccountId` or derived from `cosmrs::AccountId`.
/// - `signing_key`: The `SigningKey` derived from `cosmrs::crypto::secp256k1`. This field 
///	may not be directly used in this struct, but is available for functions that require it.
/// - `public_key`: The `PublicKey` derived from `cosmrs::crypto::secp256k1`.
pub struct Privkey {
	/// Raw byte stream of the private key, which is hex-encoded binary data.
	bytes: Vec<u8>,

	/// The private key in hexadecimal format.
	hex: String,

	/// The signing key derived from `cosmrs::crypto::secp256k1`. 
	/// This field is not used within this struct, but it may be necessary for other functions.
	#[allow(dead_code)]
	signing_key: SigningKey,

	/// The public key derived from `cosmrs::crypto::secp256k1`.
	/// This field is not used within this struct, but it may be necessary for other functions.
	#[allow(dead_code)]
	public_key: PublicKey,

	/// The `AccountID` or address associated with this private key.
	account_id: AccountId,
}

impl std::fmt::Debug for Privkey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Privkey {{ address: {} }}", self.get_address())
	}
}

impl PartialEq for Privkey {
	fn eq(&self, other: &Self) -> bool {
		self.hex.eq_ignore_ascii_case(&other.hex)
	}
}

impl Eq for Privkey {}

impl Privkey {
	/// Constructs a new `Privkey` from the provided fields.
	pub fn new(
		hex: String,
		bytes: Vec<u8>,
		signing_key: SigningKey,
		public_key: PublicKey,
		account_id: AccountId,
) -> Self {
		Self {
			hex,
			bytes,
			signing_key,
			public_key,
			account_id,
		}
	}
}

impl Privkey {

	pub fn get_address(&self) -> String {
		self.account_id.to_string()
	}

	pub fn get_hex(&self) -> String {
		self.hex.clone()
	}

	/// Saves the `bytes` (`Vec<u8>`) back to a private key file.
	///
	/// A valid file path or home directory must be provided for the file to be written.
	/// If the file already exists, the function will return an error unless the `force` parameter is set to `true`,
	/// in which case it will overwrite the existing file.
	///
	/// # Arguments
	///
	/// - `file`: An optional reference to a `Path` that specifies the location to save the private key.
	/// - `home`: An optional reference to a home directory path.
	/// - `force`: A boolean flag indicating whether to overwrite the file if it already exists.
	///
	/// # Returns
	///
	/// - `Ok(())` if the private key was successfully written to the file.
	/// - `Err(Report)` if there was an error during the save operation, such as if the file already exists and `force` is `false`.
	///
	/// # Example
	///
	/// ```
	/// let privkey = Privkey::new(Some("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"))?;
	/// let file_path = Path::new("/path/to/save/privkey_file");
	/// 
	/// // Attempt to save the private key, overwriting if necessary
	/// privkey.save(Some(file_path), None, true)?;
	/// ```
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
