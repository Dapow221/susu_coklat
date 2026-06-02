use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct JupiterClient {
    http: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuoteRequest {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    pub slippage_bps: u32,
    pub only_direct_routes: bool,
    pub restrict_intermediate_tokens: bool,
    pub max_accounts: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuoteResponse {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
}

impl JupiterClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into(),
        }
    }

    pub async fn quote(&self, request: &QuoteRequest) -> Result<QuoteResponse> {
        let url = format!("{}/swap/v1/quote", self.base_url.trim_end_matches('/'));
        let response = self
            .http
            .get(url)
            .query(&[
                ("inputMint", request.input_mint.as_str()),
                ("outputMint", request.output_mint.as_str()),
                ("amount", &request.amount.to_string()),
                ("slippageBps", &request.slippage_bps.to_string()),
                ("onlyDirectRoutes", &request.only_direct_routes.to_string()),
                (
                    "restrictIntermediateTokens",
                    &request.restrict_intermediate_tokens.to_string(),
                ),
                ("maxAccounts", &request.max_accounts.to_string()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<QuoteResponse>()
            .await?;

        Ok(response)
    }
}
