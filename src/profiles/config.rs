use crate::validators::ValidatorCollection;
use crate::validators::Validator;


#[allow(dead_code)]
pub fn default_config(profile_name: &str) -> String {
    format!(
        "PROFILE={}\n\
        MINIMUM_BALANCE=10.00\n\
        MINIMUM_BALANCE_RATIO=0.001\n\
        MINIMUM_STAKE=5\n\
        ADJUST_MINIMUM_STAKE=true\n\
        MINIMUM_STAKE_ROUNDING=5\n\
        DAILY_REWARD=0.00\n\
        read VALIDATOR MONIKER <<< \"nomic1jpvav3h0d2uru27fcne3v9k3mrl75l5zzm09uj radicalu\"\n\
        read VALIDATOR MONIKER <<< \"nomic1stfhcjgl9j7d9wzultku7nwtjd4zv98pqzjmut maximusu\"",
        profile_name
    )
}

pub struct Config {
    profile: String,
    minimum_balance: usize,
    minimum_balance_ratio: f32,
    minimum_stake: usize,
    adjust_minimum_stake: bool,
    daily_reward: f32,
    validators: ValidatorCollection,
}



//
//        PROFILE=dad
//        MINIMUM_BALANCE=10.00\n\
//        MINIMUM_BALANCE_RATIO=0.001\n\
//        MINIMUM_STAKE=5\n\
//        ADJUST_MINIMUM_STAKE=true\n\
//        MINIMUM_STAKE_ROUNDING=5\n\
//        DAILY_REWARD=0.00\n\
//
