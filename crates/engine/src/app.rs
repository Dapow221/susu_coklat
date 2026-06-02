use anyhow::Result;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use execution::{jito_bundle::JitoClient, simulator};
use jupiter_client::quote::JupiterClient;
use market_data::pool_subscriber::PoolSubscriber;
use serde::{Deserialize, Serialize};
use shared::events::EngineEvent;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::{config::Settings, state::EngineState};

pub async fn run(settings: Settings) -> Result<()> {
    let state = Arc::new(Mutex::new(EngineState {
        dry_run: settings.risk.dry_run,
        ..Default::default()
    }));
    state
        .lock()
        .await
        .push_event(EngineEvent::info("engine starting"));

    info!(
        dry_run = settings.risk.dry_run,
        http_url = %settings.rpc.http_url,
        ws_url = %settings.rpc.ws_url,
        commitment = %settings.rpc.commitment,
        poll_fallback_ms = settings.engine.poll_fallback_ms,
        "engine configured"
    );

    info!(
        only_direct_routes = settings.jupiter.only_direct_routes,
        restrict_intermediate_tokens = settings.jupiter.restrict_intermediate_tokens,
        slippage_bps = settings.jupiter.slippage_bps,
        max_accounts = settings.jupiter.max_accounts,
        "jupiter quote settings loaded"
    );

    info!(
        min_profit_bps = settings.risk.min_profit_bps,
        max_trade_lamports = settings.risk.max_trade_lamports,
        max_daily_loss_lamports = settings.risk.max_daily_loss_lamports,
        min_wallet_balance_lamports = settings.risk.min_wallet_balance_lamports,
        safety_margin_bps = settings.risk.safety_margin_bps,
        min_jito_tip_lamports = settings.jito.min_tip_lamports,
        max_jito_tip_lamports = settings.jito.max_tip_lamports,
        "risk controls loaded"
    );

    let subscriber = PoolSubscriber::new(settings.rpc.ws_url.clone(), settings.pools.clone());
    tokio::spawn(async move {
        if let Err(error) = subscriber.run().await {
            warn!(%error, "pool subscriber stopped");
        }
    });

    let _jupiter = JupiterClient::new(settings.jupiter.base_url.clone());
    let jito = JitoClient::new(settings.jito.block_engine_url.clone());

    let enabled_count = settings.pools.iter().filter(|pool| pool.enabled).count();
    if enabled_count == 0 {
        warn!("no enabled pools configured; edit config/pools.toml to start watching real pools");
    }

    info!("engine scaffold is running; real pool decoding and transaction building are TODOs");

    let _ = simulator::simulate;
    let _ = jito;

    // Background scan loop: emits heartbeats (and would emit opportunity events
    // once real pool decoding is wired in). Runs silently.
    let scan_state = state.clone();
    tokio::spawn(async move {
        run_scan_loop(scan_state, enabled_count).await;
    });

    let app = Router::new()
        .route("/status", get(status))
        .route("/pause", post(pause))
        .route("/resume", post(resume))
        .route("/events", get(events))
        .with_state(state);

    let addr: SocketAddr = settings.engine.bind_addr.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, "engine api listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn run_scan_loop(state: Arc<Mutex<EngineState>>, enabled_count: usize) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    // Burn the immediate first tick so heartbeats start 30s after boot.
    interval.tick().await;
    loop {
        interval.tick().await;
        let message = if enabled_count == 0 {
            "no pools enabled - heartbeating in idle mode".to_string()
        } else {
            format!("watching {enabled_count} pool(s)")
        };
        let mut s = state.lock().await;
        s.push_event(EngineEvent::heartbeat(message));
    }
}

#[derive(Deserialize)]
struct EventsQuery {
    since: Option<u64>,
}

#[derive(Serialize)]
struct EventsResponse {
    events: Vec<EngineEvent>,
    next_cursor: u64,
    dry_run: bool,
}

async fn events(
    State(state): State<Arc<Mutex<EngineState>>>,
    Query(query): Query<EventsQuery>,
) -> impl IntoResponse {
    let cursor = query.since.unwrap_or(0);
    let s = state.lock().await;
    let events = s.events_since(cursor);
    Json(EventsResponse {
        events,
        next_cursor: s.event_counter,
        dry_run: s.dry_run,
    })
}

async fn status(State(state): State<Arc<Mutex<EngineState>>>) -> impl IntoResponse {
    let s = state.lock().await;
    let mode = if s.paused { "paused" } else { "running" };
    let dry_run = if s.dry_run { "on" } else { "off" };
    let last_event = s
        .recent_events
        .last()
        .map(|event| event.message.as_str())
        .unwrap_or("no events");

    format!(
        "Engine status: {mode}\nDry run: {dry_run}\nEvents emitted: {}\nLast event: {last_event}",
        s.event_counter
    )
}

async fn pause(State(state): State<Arc<Mutex<EngineState>>>) -> impl IntoResponse {
    let mut state = state.lock().await;
    state.paused = true;
    state.push_event(EngineEvent::info("engine paused from api"));
    "ok"
}

async fn resume(State(state): State<Arc<Mutex<EngineState>>>) -> impl IntoResponse {
    let mut state = state.lock().await;
    state.paused = false;
    state.push_event(EngineEvent::info("engine resumed from api"));
    "ok"
}
