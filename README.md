# Solana MEV Bot Scaffold

Rust runs the latency-sensitive engine. TypeScript runs Telegram commands and config.

Default mode is `dry_run = true`. Do not put a funded private key into this project until simulation, limits, logging, and Jito delivery are tested with tiny size.

## Workflow

```txt
WebSocket per pool -> detect price gap -> threshold filter -> Jupiter direct quote
-> optimal sizing -> simulate -> Jito bundle -> Telegram alert
```

## Run

### 1. Install Requirements

```bash
rustc --version
cargo --version
node --version
npm --version
```

This scaffold was verified with Cargo `1.79.0`, Node/npm available locally, and pinned Rust dependencies in `Cargo.lock`.

### 2. Install TypeScript Dependencies

```bash
npm install
```

### 3. Configure Environment

```bash
cp .env.example .env
```

Fill:

```txt
TELEGRAM_BOT_TOKEN=your_bot_token
TELEGRAM_ADMIN_CHAT_ID=your_telegram_chat_id
ENGINE_API_URL=http://127.0.0.1:8787
```

If `TELEGRAM_ADMIN_CHAT_ID` is empty, any chat can use admin commands. Set it before running a real bot.

### 4. Configure Pools

Edit `config/pools.toml`.

Each pool starts with:

```toml
enabled = false
```

Replace the placeholder addresses and set `enabled = true` only for pools you want the engine to watch.

### 5. Start The Rust Engine

```bash
cargo run -p engine -- --config config/bot.toml --pools config/pools.toml
```

The engine exposes a local control API:

```txt
GET  http://127.0.0.1:8787/status
POST http://127.0.0.1:8787/pause
POST http://127.0.0.1:8787/resume
```

### 6. Start Telegram Bot

Open another terminal:

```bash
npm run telegram
```

Telegram commands currently included:

```txt
/start
/status
/pause
/resume
/set_threshold <bps>
/set_dry_run on|off
```

### 7. Verify Before Real Trading

```bash
cargo check
npm run typecheck
```

Keep `dry_run = true` in `config/bot.toml` while adding real pool decoding, Jupiter swap instruction building, transaction simulation, and Jito bundle sending.

## Main Files

- `config/bot.toml`: RPC, Jupiter, Jito, and risk settings.
- `config/pools.toml`: DLMM/PumpSwap pool watchlist.
- `crates/engine`: orchestrates the full loop.
- `crates/market-data`: WebSocket/pool update layer.
- `crates/strategy`: gap detection, thresholds, sizing.
- `crates/jupiter-client`: quote/swap API adapter.
- `crates/execution`: simulation, transaction, Jito bundle adapter.
- `apps/telegram-bot`: Telegram control plane.

## Current Implementation Status

Ready:

- Rust workspace and TypeScript Telegram app scaffold.
- Config files for RPC, WebSocket, Jupiter, Jito, and risk limits.
- Local engine API for status/pause/resume.
- Strategy primitives for gap detection, threshold check, and basic sizing.
- Jupiter quote client adapter.
- Jito bundle client adapter with `dry_run` guard.
- Telegram commands that call the engine API and update risk config.

Still TODO before live trading:

- Decode real Meteora DLMM and PumpSwap pool state.
- Build real Solana/Jupiter swap instructions.
- Add wallet signer loading.
- Call real `simulateTransaction` or bundle simulation.
- Add final post-simulation profit assertion.
- Send real Jito bundles only after tiny-size dry-run validation.
