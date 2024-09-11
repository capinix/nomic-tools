use crate::globals::NOMIC;

#[derive(Debug)]
pub struct Delegations {
    validator: String,
    staked: u64,
    liquid: u64,
    liquid_nbtc: u64,
    rank: u32,
    voting_power: u64,
    moniker: u64,
}

#[derive(Debug)]
struct Config {
    profile: String,
    minimum_balance: f64,
    minimum_balance_ratio: f64,
    minimum_stake: f64,
    adjust_minimum_stake: String,
    minimum_stake_rounding: f64,
	validator: String,
	validators: String,
}

#[derive(Debug)]
pub struct Profile {
    timestamp: u64,
    address: String,
    profile: String,
    balance: u64,
    balance_nbtc: u64,
    ibc_escrowed_nbtc: u64,
	delegations: Delegations,
	total_staked: u64,
	total_liquid: u64,
	total_liquid_nbtc: u64,
	config: Config,
	minimum_balance_config: u64,
	minimum_balance_ratio: f64,
	minimum_stake_config: u64,
	adjust_minimum_stake: bool,
	minimum_stake_rounding: u64,
	validator: String,
	moniker: String,
	staked: u64,
}
