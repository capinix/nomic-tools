
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

// 		println!("current_timestamp: {}", current_timestamp);
// 		println!("last_timestamp: {}", last_timestamp);
// 		println!("current_total_staked: {}", current_total_staked);
// 		println!("last_total_staked: {}", last_total_staked);
// 		println!("current_total_liquid: {}", current_total_liquid);
// 		println!("last_total_liquid: {}", last_total_liquid);
        // Calculate the daily reward
        let reward_difference = current_total_liquid - last_total_liquid;
        let time_difference = current_timestamp - last_timestamp;
        
        // Calculate the daily reward as an integer
        let daily_reward = (reward_difference as f64 / time_difference as f64 * 86400.0).round() as u64;
        daily_reward
    } else {
        0
    }
}
