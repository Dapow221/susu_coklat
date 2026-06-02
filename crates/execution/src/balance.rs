use anyhow::{bail, Result};

pub fn ensure_balance(
    balance_lamports: u64,
    needed_lamports: u64,
    min_remaining_lamports: u64,
) -> Result<()> {
    if balance_lamports < needed_lamports + min_remaining_lamports {
        bail!("insufficient balance for trade plus configured reserve");
    }
    Ok(())
}
