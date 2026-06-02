pub fn passes_threshold(estimated_net_bps: i64, min_profit_bps: i64) -> bool {
    estimated_net_bps >= min_profit_bps
}
