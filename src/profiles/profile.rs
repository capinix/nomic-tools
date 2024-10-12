
use crate::key::PrivKey;
use crate::profiles::Balance;
use crate::profiles::Delegations;
use crate::profiles::Config;
use crate::validators::ValidatorCollection;
use crate::nonce;
use eyre::eyre;
use eyre::Result;
use eyre::WrapErr;
use once_cell::sync::OnceCell;
use std::cmp::PartialEq;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;


#[derive(Clone)]
pub struct Profile {
    home:        PathBuf,
    key:         OnceCell<PrivKey>,
    config:      OnceCell<Config>,
    balance:     OnceCell<Balance>,
    delegations: OnceCell<Delegations>,
    validators:  OnceCell<ValidatorCollection>,
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
            config:      OnceCell::new(),
            balance:     OnceCell::new(),
            delegations: OnceCell::new(),
            validators:  OnceCell::new(),
        })
    }

    /// Setter for the validators field with more flexible error handling.
    pub fn set_validators(&self, validators: ValidatorCollection) -> Result<()> {
        // Try to set the validators only if it hasn't been set before.
        self.validators
            .set(validators)
            .map_err(|_| eyre!("Validators have already been set")) // Use anyhow for errors
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

    pub fn export(&self) -> Result<&str> {
        self.key()?.export()
    }

    pub fn address(&self) -> Result<&str> {
        self.key()?.address()
    }

    /// Retrieves the balance, initializing it if necessary.
    pub fn balance(&self) -> Result<&Balance> {
        self.balance.get_or_try_init(|| {
            Balance::fetch(Some(self.address()?))
        })
    }

    /// Retrieves delegations, initializing it if necessary.
    pub fn delegations(&self) -> Result<&Delegations> {
        self.delegations.get_or_try_init(|| {
            Delegations::fetch(Some(self.home()?))
        })
    }

    /// Retrieves validators, initializing it if necessary.
    pub fn validators(&self) -> Result<&ValidatorCollection> {
        self.validators.get_or_try_init(|| {
            ValidatorCollection::fetch()
        })
    }

    /// Retrieves config, initializing it if necessary.
    pub fn config(&self) -> Result<&Config> {
        self.config.get_or_try_init(|| {
            Config::load(self.config_file()?, true)
        })
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
