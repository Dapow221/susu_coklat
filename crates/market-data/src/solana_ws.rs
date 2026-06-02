use anyhow::Result;
use shared::types::PoolConfig;
use tracing::info;

#[derive(Debug, Clone)]
pub struct SolanaWsClient {
    ws_url: String,
}

impl SolanaWsClient {
    pub fn new(ws_url: impl Into<String>) -> Self {
        Self {
            ws_url: ws_url.into(),
        }
    }

    pub async fn subscribe_pool_accounts(&self, pools: &[PoolConfig]) -> Result<()> {
        info!(
            ws_url = %self.ws_url,
            pool_count = pools.len(),
            "starting websocket account subscriptions"
        );

        // TODO: connect with tokio_tungstenite and send accountSubscribe per pool.
        // Keep decoding separated by pool kind so DLMM/PumpSwap parsers stay testable.
        Ok(())
    }
}
