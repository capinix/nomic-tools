use crate::profiles::Profile;
use crate::profiles::ProfileCollection;


pub struct Calc {
    pub available: u64,
}


impl Calc {
    // Optional: Add more functionality to Calc if needed
    pub fn new(available: u64) -> Self {
        Self { available }
    }
}


impl Profile {
    pub fn available_balance(&self) -> eyre::Result<u64> {
        let available_balance = self.balance()?.nom + self.delegations()?.total().liquid;
        Ok(available_balance)
    }
    pub fn total_staked(&self) -> eyre::Result<u64> {
        let total_staked = self.delegations()?.total().staked;
        Ok(total_staked)
    }
    pub fn calculated_minimum_balance(&self) -> eyre::Result<u64> {
        let total_staked = self.total_staked()?;
        let balance_ratio = self.config()?.minimum_balance_ratio;
        let current_minimum_balance = self.config()?.minimum_balance;
        let minimum_balance = (total_staked as f64 * balance_ratio).round() as u64;
        Ok(max(minimum_balance, current_minimum_balance))
    }
}

