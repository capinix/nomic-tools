
use eyre::ContextCompat;
use eyre::WrapErr;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::fs;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConfigValidator {
    pub address: String,
    pub moniker: String,
}

impl ConfigValidator {
    pub fn new(address: String, moniker: String) -> Self {
        Self { address, moniker }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub profile: String,
    pub minimum_balance: u64,
    pub minimum_balance_ratio: f32,
    pub minimum_stake: u64,
    pub adjust_minimum_stake: bool,
    pub daily_reward: f64,
    pub validators: Vec<ConfigValidator>,
}

impl Config {
    pub fn new(profile: String) -> Self {
        Self {
            profile,
            minimum_balance: 0,
            minimum_balance_ratio: 1.0,
            minimum_stake: 0,
            adjust_minimum_stake: false,
            daily_reward: 0.0,
            validators: Vec::new(),
        }
    }

    pub fn add_validator(&mut self, address: String, moniker: String) {
        self.validators.push(ConfigValidator::new(address, moniker));
    }

    pub fn default(profile: String) -> Self {
        let validators = vec![
            ConfigValidator::new(
                "nomic1jpvav3h0d2uru27fcne3v9k3mrl75l5zzm09uj".to_string(),
                "radicalu".to_string(),
            ),
            ConfigValidator::new(
                "nomic1stfhcjgl9j7d9wzultku7nwtjd4zv98pqzjmut".to_string(),
                "maximusu".to_string(),
            ),
        ];
        Self {
            profile,
            minimum_balance: 100_000,
            minimum_balance_ratio: 0.001,
            minimum_stake: 200_000,
            adjust_minimum_stake: true,
            daily_reward: 0.0,
            validators,
        }
    }

    pub fn rotate_validators(&mut self) -> &mut Self {
        if let Some(last) = self.validators.pop() {
            self.validators.insert(0, last);
        }
        self
    }

    pub fn import(config_str: &str) -> eyre::Result<Self> {
        let mut config_map = HashMap::new();
        let mut validators = Vec::new();
        let comment_regex = Regex::new(r"^\s*#").unwrap();
        let read_validator_regex = Regex::new(concat!(
            r"^\s*read\s+(?:-r\s+)?VALIDATOR\s+MONIKER\s+<<<\s*",
            r#"["]?([^\s"']+)["]?\s+["]?([^"\s]+)["]?"#
        )).unwrap();

        for line in config_str.lines() {
            let line = line.trim();

            if line.is_empty() || comment_regex.is_match(line) {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                config_map.insert(key.trim(), value.trim().to_string());
            } else if let Some(captures) = read_validator_regex.captures(line) {
                let address = captures.get(1).context("Missing validator address")?.as_str().to_string();
                let moniker = captures.get(2).context("Missing validator moniker")?.as_str().to_string();
                validators.push(ConfigValidator { address, moniker });
            }
        }

        Ok(Config {
            profile: config_map.get("PROFILE").context("Missing profile")?.clone(),
            minimum_balance: (config_map.get("MINIMUM_BALANCE")
                .context("Missing minimum balance")?
                .parse::<f64>()
                .context("Invalid minimum balance")? * 1_000_000.0) as u64,
            minimum_balance_ratio: config_map.get("MINIMUM_BALANCE_RATIO")
                .context("Missing minimum balance ratio")?
                .parse::<f32>()
                .context("Invalid minimum balance ratio")?,
            minimum_stake: (config_map.get("MINIMUM_STAKE")
                .context("Missing minimum stake")?
                .parse::<f64>()
                .context("Invalid minimum stake")? * 1_000_000.0) as u64,
            adjust_minimum_stake: config_map.get("ADJUST_MINIMUM_STAKE")
                .context("Missing adjust minimum stake")?
                .parse::<bool>()
                .context("Invalid adjust minimum stake")?,
            daily_reward: config_map.get("DAILY_REWARD")
                .context("Missing daily reward")?
                .parse::<f64>()
                .context("Invalid daily reward")?,
            validators,
        })
    }

    pub fn load<P: AsRef<Path>>(path: P, new: bool) -> eyre::Result<Self> {
        let path = path.as_ref();

        if path.exists() {
            let file_content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read from file: {:?}", path))?;
            Self::import(&file_content)
                .context("Failed to import configuration from file content")
        } else if new {
            let default_profile = "default_profile".to_string();
            Ok(Self::default(default_profile))
        } else {
            Err(eyre::eyre!("Configuration file not found and 'new' is false"))
        }
    }

    pub fn validator(&self) -> eyre::Result<String> {
        self.validators.last()
            .map(|validator| validator.address.clone())
            .ok_or_else(|| eyre::eyre!("No validators found"))
    }

    pub fn export(&self) -> eyre::Result<String> {
        let mut output = String::new();
        output.push_str(&format!("PROFILE={}\n", self.profile));
        output.push_str(&format!("MINIMUM_BALANCE={:.2}\n", self.minimum_balance as f64 / 1_000_000.0));
        output.push_str(&format!("MINIMUM_BALANCE_RATIO={:.3}\n", self.minimum_balance_ratio));
        output.push_str(&format!("MINIMUM_STAKE={:.2}\n", self.minimum_stake as f64 / 1_000_000.0));
        output.push_str(&format!("ADJUST_MINIMUM_STAKE={}\n", self.adjust_minimum_stake));
        output.push_str(&format!("DAILY_REWARD={:.2}\n", self.daily_reward));

        for validator in &self.validators {
            output.push_str(&format!(
                "read -r VALIDATOR MONIKER <<< \"{} {}\"\n",
                validator.address, validator.moniker
            ));
        }

        Ok(output)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P, overwrite: bool) -> eyre::Result<()> {
        let path = path.as_ref();

        if path.exists() && !overwrite {
            return Err(eyre::eyre!("File already exists at {:?} and overwrite is not allowed.", path));
        }

        let file = File::create(path).map_err(|e| eyre::eyre!("Failed to create file: {}", e))?;
        let mut writer = BufWriter::new(file);

        let config_str = self.export()?;
        writer.write_all(config_str.as_bytes())
            .with_context(|| format!("Failed to write to file: {:?}", path))?;

        Ok(())
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Call the export method and write its result to the Debug output
        match self.export() {
            Ok(exported_string) => f.write_str(&exported_string),
            Err(e) => f.write_str(&format!("Error exporting: {}", e)),
        }
    }
}
