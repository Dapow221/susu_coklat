pub fn bps_to_lamports(amount_lamports: u64, bps: i64) -> i64 {
    ((amount_lamports as i128 * bps as i128) / 10_000) as i64
}
