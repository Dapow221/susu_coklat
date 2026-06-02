use shared::types::{Opportunity, PoolPrice};

use crate::threshold::passes_threshold;

pub fn detect(
    prices: &[PoolPrice],
    min_profit_bps: i64,
    safety_margin_bps: i64,
) -> Vec<Opportunity> {
    let mut opportunities = Vec::new();

    for buy in prices {
        for sell in prices {
            if buy.pool.address == sell.pool.address {
                continue;
            }

            if buy.pool.base_mint != sell.pool.base_mint
                || buy.pool.quote_mint != sell.pool.quote_mint
            {
                continue;
            }

            if buy.price <= 0.0 || sell.price <= 0.0 || sell.price <= buy.price {
                continue;
            }

            let gross_gap_bps = (((sell.price / buy.price) - 1.0) * 10_000.0) as i64;
            let costs_bps = buy.pool.fee_bps as i64
                + sell.pool.fee_bps as i64
                + buy.pool.bin_step_bps as i64
                + sell.pool.bin_step_bps as i64
                + safety_margin_bps;
            let estimated_net_bps = gross_gap_bps - costs_bps;

            if passes_threshold(estimated_net_bps, min_profit_bps) {
                opportunities.push(Opportunity {
                    buy_pool: buy.pool.clone(),
                    sell_pool: sell.pool.clone(),
                    gross_gap_bps,
                    estimated_net_bps,
                });
            }
        }
    }

    opportunities.sort_by(|a, b| b.estimated_net_bps.cmp(&a.estimated_net_bps));
    opportunities
}
