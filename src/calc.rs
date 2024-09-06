
pub fn daily_reward(
    current_timestamp: u64,
    current_total_staked: u64,
    current_total_liquid: u64,
    last_timestamp: u64,
    last_total_staked: u64,
    last_total_liquid: u64,
) -> u64 {
    // Check if the conditions are met
    if last_total_staked == current_total_staked
        && last_total_liquid < current_total_liquid
        && last_timestamp < current_timestamp
        && last_timestamp > 0
    {
        // Calculate the deltas
        let reward_delta = current_total_liquid - last_total_liquid;
        let time_delta = current_timestamp - last_timestamp;
        
        // Calculate the daily reward as an integer
        let daily_reward = (reward_delta as f64 / time_delta as f64 * 86400.0).round() as u64;

        daily_reward
    } else {
        0
    }
}

pub fn minimum_balance(
    total_staked: u64,
    balance_ratio: f64,
    current_minimum_balance: u64,
) -> u64 {
        let minimum_balance = (total_staked as f64 * balance_ratio).round() as u64;
		if minimum_balance < current_minimum_balance {
			current_minimum_balance 
		} else {
			minimum_balance
		}
}
