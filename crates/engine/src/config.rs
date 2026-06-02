use anyhow::Result;
use serde::Deserialize;
use shared::types::PoolConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub rpc: RpcConfig,
    pub jupiter: JupiterConfig,
    pub jito: JitoConfig,
    pub risk: RiskConfig,
    pub engine: EngineConfig,
    pub pools: Vec<PoolConfig>,
}

#[derive(Debug, Clone, Deserialize)]
struct BotSettings {
    rpc: RpcConfig,
    jupiter: JupiterConfig,
    jito: JitoConfig,
    risk: RiskConfig,
    engine: EngineConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcConfig {
    pub http_url: String,
    pub ws_url: String,
    pub commitment: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterConfig {
    pub base_url: String,
    pub only_direct_routes: bool,
    pub restrict_intermediate_tokens: bool,
    pub slippage_bps: u32,
    pub max_accounts: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JitoConfig {
    pub block_engine_url: String,
    pub min_tip_lamports: u64,
    pub max_tip_lamports: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RiskConfig {
    pub dry_run: bool,
    pub min_profit_bps: i64,
    pub max_trade_lamports: u64,
    pub max_daily_loss_lamports: u64,
    pub min_wallet_balance_lamports: u64,
    pub safety_margin_bps: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfig {
    pub bind_addr: String,
    pub poll_fallback_ms: u64,
}

#[derive(Debug, Deserialize)]
struct PoolsFile {
    pools: Vec<PoolConfig>,
}

impl Settings {
    pub fn load(bot_path: &str, pools_path: &str) -> Result<Self> {
        let bot: BotSettings = config::Config::builder()
            .add_source(config::File::with_name(bot_path))
            .build()?
            .try_deserialize()?;

        let pools_raw = std::fs::read_to_string(pools_path)?;
        let pools: PoolsFile = toml::from_str(&pools_raw)?;

        Ok(Settings {
            rpc: bot.rpc,
            jupiter: bot.jupiter,
            jito: bot.jito,
            risk: bot.risk,
            engine: bot.engine,
            pools: pools.pools,
        })
    }
}
