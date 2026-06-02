use anyhow::Result;
use shared::types::PoolConfig;

use crate::solana_ws::SolanaWsClient;

pub struct PoolSubscriber {
    ws: SolanaWsClient,
    pools: Vec<PoolConfig>,
}

impl PoolSubscriber {
    pub fn new(ws_url: impl Into<String>, pools: Vec<PoolConfig>) -> Self {
        Self {
            ws: SolanaWsClient::new(ws_url),
            pools,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let enabled: Vec<PoolConfig> = self
            .pools
            .iter()
            .filter(|pool| pool.enabled)
            .cloned()
            .collect();

        self.ws.subscribe_pool_accounts(&enabled).await
    }
}
