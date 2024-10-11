
use crate::nonce;
use crate::key::PrivKey;
use eyre::{Result, WrapErr};
use once_cell::sync::OnceCell;
use orga::client::wallet::SimpleWallet;
//use orga::client::wallet::Wallet;
//use orga::coins::Address;
//use orga::secp256k1::SecretKey;
//use rand::Rng;
use std::cmp::PartialEq;
use std::{
    fs,
    fs::File,
    io::Read,
    path::Path,
    path::PathBuf,
};

#[derive(Clone)]
pub struct Profile {
    home:        PathBuf,
    key:         OnceCell<PrivKey>,
    wallet:      OnceCell<SimpleWallet>,
}

impl Profile {
    pub fn new<P: AsRef<Path>>(home: P) -> Result<Self> {

        let home = home.as_ref().to_path_buf();

        // Create the home directory
        fs::create_dir_all(&home)
            .wrap_err_with(|| format!("Failed to create directory: {:?}", home))?;

        Ok(Self {
            home,
            key:         OnceCell::new(),
            wallet:      OnceCell::new(),
        })
    }

    pub fn home(&self) -> Result<&Path> {
        Ok(&self.home)
    }

    pub fn name(&self) -> Result<&str> {
        self.home
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| eyre::eyre!("Failed to extract the last component of the path"))
    }

    pub fn wallet_path(&self) -> Result<PathBuf> {
        let wallet_path = self.home.join(".orga-wallet");
        fs::create_dir_all(&wallet_path)
            .wrap_err_with(|| format!("Failed to create directory: {:?}", wallet_path))?;
        Ok(wallet_path)
    }

    /// Returns the config file path.
    pub fn config_file(&self) -> Result<PathBuf> {
        Ok(self.home.join("config"))
    }

    /// Returns the key file path.
    pub fn key_file(&self) -> Result<PathBuf> {
        let wallet_path = self.wallet_path()?;
        Ok(wallet_path.join("privkey"))
    }

    /// Returns the nonce file path.
    pub fn nonce_file(&self) -> Result<PathBuf> {
            Ok(self.wallet_path()?.join("nonce"))
    }

    pub fn key(&self) -> Result<&PrivKey> {
        self.key.get_or_try_init(|| {
            PrivKey::load(self.key_file()?, true)
        })
    }

    #[allow(dead_code)]
    pub fn wallet(&self) -> Result<&SimpleWallet> {
        self.wallet.get_or_try_init(|| {
            SimpleWallet::open(self.wallet_path()?)
                .map_err(|e| eyre::eyre!("Failed to open wallet: {}", e))
        })
    }

    pub fn export(&self) -> Result<&str> {
        self.key()?.export()
    }

    pub fn address(&self) -> Result<&str> {
        self.key()?.address()
    }

    /// reads and returns the content of the config file.
    pub fn config(&self) -> Result<String> {
        let config_file = self.config_file()?;
        // attempt to open the config file
        let mut file = File::open(config_file.clone())
            .with_context(|| format!("failed to open config file at {:?}", config_file))?;

        // read the file content into a string
        let mut content = String::new();
        file.read_to_string(&mut content)
            .with_context(|| format!("failed to read config file at {:?}", config_file))?;

        Ok(content)
    }

    pub fn import(&self, hex_str: &str, force: bool) -> Result<()> {
        let key_file = self.key_file()?; // Get the key file path

        // Check if the key file already exists
        if key_file.exists() && !force {
            return Err(eyre::eyre!("Key file already exists. Use 'force' to overwrite it."));
        }

        // Import the private key from the hex string
        let key = PrivKey::import(hex_str)?;

        // Save the key to the key file
        key.save(key_file, force)?;

        Ok(())
    }

    pub fn export_nonce(&self) -> Result<u64> {
        let nonce_file = self.nonce_file()?;
        nonce::export(Some(&nonce_file), None)
    }

    pub fn import_nonce(&self, value: u64, dont_overwrite: bool) -> Result<()> {
        let nonce_file = self.nonce_file()?;
        nonce::import(value, Some(&nonce_file), None, dont_overwrite)
    }
}

// Custom Debug implementation for Profile
impl std::fmt::Debug for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Handle the address and name Results manually, converting them into debug-friendly formats
        let address_str = match self.address() {
            Ok(address) => address.to_string(),             // Convert &str to String for debug printing
            Err(_) => "Error fetching address".to_string(), // Handle error case
        };

        let name_str = match self.name() {
            Ok(name) => name.to_string(),                // Convert &str to String
            Err(_) => "Error fetching name".to_string(), // Handle error case
        };

        write!(
            f,
            "Profile {{ address: {}, name: {} }}",
            address_str, name_str
        )
    }
}

impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        // Compare the addresses of the two profiles
        match (self.address(), other.address()) {
            (Ok(addr1), Ok(addr2)) => addr1 == addr2, // Both addresses are valid
            _ => false, // If either address is an error, they're not equal
        }
    }
}
