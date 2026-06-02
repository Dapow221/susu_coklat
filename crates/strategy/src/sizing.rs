use shared::types::{Opportunity, TradePlan};

pub fn size(opportunity: Opportunity, max_trade_lamports: u64) -> TradePlan {
    let expected_profit_lamports =
        ((max_trade_lamports as i128 * opportunity.estimated_net_bps as i128) / 10_000) as i64;

    TradePlan {
        opportunity,
        amount_lamports: max_trade_lamports,
        expected_profit_lamports,
    }
}
