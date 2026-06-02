use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PoolKind {
    Dlmm,
    Pumpswap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub name: String,
    pub kind: PoolKind,
    pub address: String,
    pub base_mint: String,
    pub quote_mint: String,
    pub fee_bps: u32,
    pub bin_step_bps: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolPrice {
    pub pool: PoolConfig,
    pub price: f64,
    pub slot: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    pub buy_pool: PoolConfig,
    pub sell_pool: PoolConfig,
    pub gross_gap_bps: i64,
    pub estimated_net_bps: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradePlan {
    pub opportunity: Opportunity,
    pub amount_lamports: u64,
    pub expected_profit_lamports: i64,
}
