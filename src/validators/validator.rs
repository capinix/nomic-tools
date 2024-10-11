
// use std::mem::size_of;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Validator {
    rank         : usize,
    address      : String,
    voting_power : usize,
    moniker      : String,
    details      : String,
}

impl Validator {
    pub fn new(
        rank         : usize,
        address      : String,
        voting_power : usize,
        moniker      : String,
        details      : String
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
    pub fn rank(&self) -> usize {
        self.rank
    }

    // Getter for address
    pub fn address(&self) -> &str {
        &self.address
    }

    // Getter for voting power
    pub fn voting_power(&self) -> usize {
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
        format!("{:.2}", self.voting_power as f64 / 1_000_000.0)
    }
}
