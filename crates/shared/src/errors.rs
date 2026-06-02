use thiserror::Error;

#[derive(Debug, Error)]
pub enum BotError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("market data error: {0}")]
    MarketData(String),
    #[error("quote error: {0}")]
    Quote(String),
    #[error("execution error: {0}")]
    Execution(String),
}
