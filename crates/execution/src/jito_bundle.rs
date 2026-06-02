use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone)]
pub struct JitoClient {
    http: reqwest::Client,
    block_engine_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SendBundleRequest {
    pub transactions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendBundleResponse {
    pub result: Option<String>,
}

impl JitoClient {
    pub fn new(block_engine_url: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            block_engine_url: block_engine_url.into(),
        }
    }

    pub async fn send_bundle(
        &self,
        request: SendBundleRequest,
        dry_run: bool,
    ) -> Result<Option<String>> {
        if dry_run {
            info!(
                tx_count = request.transactions.len(),
                "dry run enabled; not sending Jito bundle"
            );
            return Ok(None);
        }

        let url = format!(
            "{}/api/v1/bundles",
            self.block_engine_url.trim_end_matches('/')
        );
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendBundle",
            "params": [request.transactions]
        });

        let response = self
            .http
            .post(url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<SendBundleResponse>()
            .await?;

        Ok(response.result)
    }
}
