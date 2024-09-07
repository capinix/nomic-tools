
pub fn stake(
    liquid: u64,
    balance: u64,
    claim_fee: u64,
    stake_fee: u64,
    minimum_balance: u64,
    staked: u64,
    minimum_stake: u64,
    adjust: bool,
    daily_reward: u64,
    rounding: f64,
) -> (bool, u64) {
    
    // Define the closure
    let calculate_quantity = |available: u64, alignment: u64, quantum: u64| -> u64 {
        if available > alignment {
            (((available - alignment) / quantum) * quantum) + alignment
        } else {
            0
        }
    };

    // Define stake_quantum
    let stake_quantum = if adjust && daily_reward > 0 {
        let rounding_unom = (rounding * 1_000_000.0) as u64;
        (daily_reward / rounding_unom) * rounding_unom
    } else {
        minimum_stake
    };

    // Calculate quantities
    let quantity_without_claim = calculate_quantity(
        balance - minimum_balance - stake_fee,
        staked % stake_quantum,
        stake_quantum,
    );

    let quantity_after_claim = calculate_quantity(
        liquid + balance - minimum_balance - claim_fee - stake_fee,
        staked % stake_quantum,
        stake_quantum,
    );

    // Return results based on the computed quantities
    if quantity_without_claim > 0 {
        return (false, quantity_without_claim);
    }

    if quantity_after_claim > 0 {
        return (true, quantity_after_claim);
    }
    
    (false, 0)
}
