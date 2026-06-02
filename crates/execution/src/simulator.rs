use anyhow::Result;
use shared::types::TradePlan;

#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub profitable: bool,
    pub expected_profit_lamports: i64,
    pub reason: String,
}

pub async fn simulate(plan: &TradePlan) -> Result<SimulationResult> {
    // TODO: build the real Solana transaction and call simulateTransaction.
    Ok(SimulationResult {
        profitable: plan.expected_profit_lamports > 0,
        expected_profit_lamports: plan.expected_profit_lamports,
        reason: "stub simulation from strategy estimate".to_string(),
    })
}
