use anyhow::Result;
use clap::Parser;

mod app;
mod config;
mod state;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = "config/bot.toml")]
    config: String,
    #[arg(long, default_value = "config/pools.toml")]
    pools: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("engine=info,market_data=info,execution=info")
        .init();

    let args = Args::parse();
    let settings = config::Settings::load(&args.config, &args.pools)?;
    app::run(settings).await
}
