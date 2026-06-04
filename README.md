# Solana MEV Bot Scaffold

Rust runs the latency-sensitive engine. TypeScript runs the Telegram notifier.

Default mode is `dry_run = true`. Do not put a funded private key into this project until simulation, limits, logging, and Jito delivery are tested with tiny size.

## Architecture

The bot runs in two layers:

1. **Rust engine** (`crates/engine`) — opens the WSS connection to your RPC, watches enabled pools, detects price gaps, sizes trades, and (in dry-run mode) reports what it would have done. Exposes a small HTTP control API on `127.0.0.1:8787`.
2. **Telegram bot** (`apps/telegram-bot`) — runs in the background, polls the engine for new events every 2s, and forwards them to the admin chat. Exposes only `/status` and `/set_dry_run` as user commands; everything else happens silently.

The engine is the source of truth. The bot is a thin notifier.

## Workflow

```
WebSocket per pool
  -> detect price gap
    -> threshold filter
      -> Jupiter direct quote
        -> optimal sizing
          -> simulate
            -> Jito bundle
              -> Telegram alert
```

## What this bot is

This scaffold is a Solana **sandwich entry** tool. It watches volume on Pump.fun token launches and Meteora DLMM pools, and when it spots a large pending entry that would move price, it prepares a Jito bundle (front-run → victim → back-run) to capture the spread.

**Status:** scaffold only. The `dry_run` guard on the Jito client is in place, but the actual victim-target detector, mempool / pending-bundle listener, and front-run/back-run bundle construction are not implemented yet (see TODO list below). Do not load a funded wallet or flip `dry_run = false` until the attack pipeline is wired up and tested with tiny size.

## Run

### 1. Install Requirements

```bash
rustc --version   # need 1.79.0 or newer
cargo --version
node --version    # need 18+ (tested on 23.1.0)
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

Fill in `.env`:

```
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

### 5. Configure RPC

Edit `config/bot.toml` under `[rpc]`. The defaults are:

```toml
[rpc]
http_url = "https://pump.helius-rpc.com/"
ws_url = "https://pump.helius-rpc.com/"
commitment = "processed"
```

For Pump.fun trading, point the RPC URLs at Helius's public Pump.fun endpoint: `https://pump.helius-rpc.com/`. For other DEXes, use your preferred HTTP/WSS RPC provider.

### 6. Start the Rust Engine

```bash
cargo run -p engine -- --config config/bot.toml --pools config/pools.toml
```

The engine exposes a local control API:

```
GET  http://127.0.0.1:8787/status
GET  http://127.0.0.1:8787/events?since=N
POST http://127.0.0.1:8787/pause
POST http://127.0.0.1:8787/resume
```

### 7. Start Telegram Bot

Open another terminal:

```bash
npm run telegram
```

The bot runs silently and forwards engine events to your admin chat in UTC. The only user commands are:

```
/start        - show welcome message
/status       - print engine status
/set_dry_run on|off
```

### 8. Verify Before Real Trading

```bash
cargo check
npm run typecheck
```

Keep `dry_run = true` in `config/bot.toml` while adding real pool decoding, Jupiter swap instruction building, transaction simulation, and Jito bundle sending.

## VPS Deployment

Quick recipe for a fresh Ubuntu 22.04+ VPS:

```bash
# 1. System packages
sudo apt update && sudo apt install -y build-essential pkg-config libssl-dev git ufw curl
sudo ufw allow OpenSSH
sudo ufw enable

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# 3. Install Node.js 20 LTS
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# 4. Clone and set up
git clone https://github.com/Dapow221/susu_coklat.git
cd susu_coklat
npm install
cp .env.example .env
nano .env                            # paste your TELEGRAM_BOT_TOKEN and TELEGRAM_ADMIN_CHAT_ID
nano config/bot.toml                 # (optional) switch ws_url to your preferred provider
nano config/pools.toml               # (optional) add real pool addresses; leave disabled for now

# 5. Build the engine
cargo build --release

# 6. Start in the background with nohup
nohup ./target/release/engine \
  --config config/bot.toml \
  --pools config/pools.toml \
  > /var/log/mev-engine.log 2>&1 &

nohup npm run telegram > /var/log/mev-bot.log 2>&1 &

# 7. Watch logs
tail -f /var/log/mev-engine.log
tail -f /var/log/mev-bot.log
```

### Running as systemd services (more robust)

Create `/etc/systemd/system/susu-engine.service`:

```ini
[Unit]
Description=Susu Coklat Solana Engine
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/susu_coklat
ExecStart=/home/ubuntu/susu_coklat/target/release/engine \
  --config /home/ubuntu/susu_coklat/config/bot.toml \
  --pools /home/ubuntu/susu_coklat/config/pools.toml
Restart=always
RestartSec=5
StandardOutput=append:/var/log/mev-engine.log
StandardError=append:/var/log/mev-engine.log

[Install]
WantedBy=multi-user.target
```

And `/etc/systemd/system/susu-bot.service`:

```ini
[Unit]
Description=Susu Coklat Telegram Bot
After=network.target susu-engine.service

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/susu_coklat
EnvironmentFile=/home/ubuntu/susu_coklat/.env
ExecStart=/usr/bin/npm run telegram
Restart=always
RestartSec=5
StandardOutput=append:/var/log/mev-bot.log
StandardError=append:/var/log/mev-bot.log

[Install]
WantedBy=multi-user.target
```

Then enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now susu-engine susu-bot
sudo systemctl status susu-engine susu-bot
sudo journalctl -u susu-engine -f
```

### Updating after code changes

```bash
cd ~/susu_coklat
git pull
npm install              # in case deps changed
cargo build --release
sudo systemctl restart susu-engine susu-bot
```

## Main Files

- `config/bot.toml`: RPC, Jupiter, Jito, and risk settings.
- `config/pools.toml`: DLMM/PumpSwap pool watchlist.
- `crates/engine`: orchestrates the full loop.
- `crates/market-data`: WebSocket/pool update layer.
- `crates/strategy`: gap detection, thresholds, sizing.
- `crates/jupiter-client`: quote/swap API adapter.
- `crates/execution`: simulation, transaction, Jito bundle adapter.
- `apps/telegram-bot`: Telegram notifier.

## Current Implementation Status

Ready:

- Rust workspace and TypeScript Telegram app scaffold.
- Config files for RPC, WebSocket, Jupiter, Jito, and risk limits.
- Local engine API for status/pause/resume/events.
- Strategy primitives for gap detection, threshold check, and basic sizing.
- Jupiter quote client adapter.
- Jito bundle client adapter with `dry_run` guard.
- Background scan loop emitting heartbeats every 30s.
- Background alert poller that forwards engine events to Telegram in UTC.
- Stripped Telegram command surface (only `/status` and `/set_dry_run` exposed).

Still TODO before live trading:

- Decode real Meteora DLMM and PumpSwap pool state.
- Build real Solana/Jupiter swap instructions.
- Add wallet signer loading.
- Call real `simulateTransaction` or bundle simulation.
- Add final post-simulation profit assertion.
- Send real Jito bundles only after tiny-size dry-run validation.

## Security Notes

- **Never** commit `.env` or any file containing your Telegram bot token or RPC key. The repo's `.gitignore` already excludes `.env`.
- If you leak a token (e.g. in a chat), **revoke it immediately** at `@BotFather` (for Telegram) or in your RPC provider's dashboard. The leaked token is permanently in any history it touched.
- The scaffold has no wallet signer loaded, so even in non-dry-run mode, the bot cannot broadcast a sandwich yet. **Do not** add a funded key until the attack pipeline is fully wired and tested with tiny-size dry-runs.
- **Ethical / legal warning:** sandwich attacks directly extract value from identified victims on every trade — the victim loses money on slippage that the bot captures. This is widely considered predatory in the Solana community. MEV front-running is also restricted or illegal in several jurisdictions under traditional finance analogues; check your local rules before deploying with real funds.
