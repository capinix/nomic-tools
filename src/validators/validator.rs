use crate::functions::format_to_millions;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Validator {
    rank: u64,
    address: String,
    voting_power: u64,
    moniker: String,
    details: String,
}

#[derive(Clone, tabled::Tabled)]
pub struct ValidatorTableDetail {
    #[tabled(rename = "Rank")]
    pub rank: u64,

    #[tabled(rename = "Validator Address")]
    pub address: String,

    #[tabled(rename = "Voting\n Power")]
    pub voting_power: String,

    #[tabled(rename = "Moniker")]
    pub moniker: String,

    #[tabled(rename = "Details")]
    pub details: String,
}

#[derive(Clone, tabled::Tabled)]
pub struct ValidatorTableSimple {
    #[tabled(rename = "Rank")]
    pub rank: u64,

    #[tabled(rename = "Validator Address")]
    pub address: String,

    #[tabled(rename = "Voting\n Power")]
    pub voting_power: String,

    #[tabled(rename = "Moniker")]
    pub moniker: String,
}


impl Validator {
    pub fn new(
        rank: u64,
        address: String,
        voting_power: u64,
        moniker: String,
        details: String,
    ) -> Self {
        Self {
            rank,
            address,
            voting_power,
            moniker,
            details,
        }
    }

    // Getter for rank
    pub fn rank(&self) -> u64 {
        self.rank
    }

    // Getter for address
    pub fn address(&self) -> &str {
        &self.address
    }

    // Getter for voting power
    pub fn voting_power(&self) -> u64 {
        self.voting_power
    }

    // Getter for moniker
    pub fn moniker(&self) -> &str {
        &self.moniker
    }

    // Getter for details
    pub fn details(&self) -> &str {
        &self.details
    }

    pub fn voting_power_nom(&self) -> String {
        // Converts voting power to NOM (e.g., from uNOM to NOM)
        format_to_millions(self.voting_power, Some(0))
    }

    pub fn table_detail(&self) -> ValidatorTableDetail {
        ValidatorTableDetail {
            rank: self.rank,
            address: self.address.clone(),
            voting_power: self.voting_power_nom(),
            moniker: self.moniker.clone(),
            details: self.details.clone(),
        }
    }

    pub fn table_simple(&self) -> ValidatorTableSimple {
        ValidatorTableSimple {
            rank: self.rank,
            address: self.address.clone(),
            voting_power: self.voting_power_nom(),
            moniker: self.moniker.clone(),
        }
    }
}

