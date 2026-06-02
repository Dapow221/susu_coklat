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

## What happens, step by step

Let's say someone launches a new token on Pump.fun: **"PEPEZILLA"**, mint `7xKDR...rQ2`. (Not a real token, just for the example.) It graduates to PumpSwap when its bonding curve fills.

**The setup you'd do:**
- Add a pool that *contains* PEPEZILLA to `config/pools.toml`. The pool address, base mint, and quote mint come from the on-chain pool account you'd fetch from the PumpSwap program.
- Set `enabled = true`.

**Now a buyer does this:** someone swaps **0.5 SOL → PEPEZILLA** on Raydium/PumpSwap/Jupiter.

**What happens, step by step:**

1. **t+0ms** — The buyer's signed tx lands in the mempool (or in Jito's pending-bundle queue).
2. **t+5–50ms** — The tx is being processed. The pool reserves shift: WSOL goes down, PEPEZILLA goes down. The on-chain pool account's data changes.
3. **t+10–100ms** — Your bot's WSS `accountSubscribe` callback fires. The engine reads the new reserves and recomputes spot price.
   - Old price: 1 PEPEZILLA = 0.0000001234 SOL
   - New price: 1 PEPEZILLA = 0.0000001267 SOL (price went up because the buyer took some PEPEZILLA out of the pool)
4. **Gap check** — The bot asks Jupiter: "what's the aggregated PEPEZILLA/SOL price right now across all pools?" Say Jupiter says `0.0000001240`. The on-chain pool is at `0.0000001267`, so the pool is **rich in WSOL** (cheaper to buy PEPEZILLA elsewhere, more profitable to sell PEPEZILLA here, or arbitrage by buying cheap on Jupiter and selling here). The gap is `(1267 - 1240) / 1240 = ~2.18% = 218 bps`, well above `min_profit_bps = 20`.
5. **Sizing** — Bot picks a trade size, say **0.05 SOL** worth of PEPEZILLA buy on this pool. Below `max_trade_lamports` (0.1 SOL), leaves profit on the table for size, etc.
6. **Simulate** — Bot builds a VersionedTransaction (buy PEPEZILLA on this pool via Jupiter's swap instruction), calls `simulateTransaction`. If it reverts or profit < 0 after slippage, abort.
7. **Jito bundle** — Bot wraps its tx in a Jito bundle with a tip (e.g. 10,000 lamports). Sends to `https://mainnet.block-engine.jito.wtf` (from `config/bot.toml`).
8. **Landing** — A Jito validator includes the bundle at the top of the next block. The bot's tx lands *before* any subsequent trades that would've closed the gap.
9. **Telegram** — Bot sends: `🔔 Arb filled. +0.00012 SOL profit. Tx: 5xK...` with timestamps displayed as UTC, for example `2026-06-02 22:56:11 UTC`.

**Where the profit comes from:** the bot captured the price difference between *this pool's microsecond-old price* and Jupiter's aggregated price. The next trader who comes in pays a slightly worse price, which is roughly the bot's profit.

**What this scaffold actually does today:** the loop is wired up but `real pool decoding and transaction building are TODOs` (per the engine's own log message). So the WebSocket → gap logic chain works at the macro level, but the bot can't actually fill any trades yet — there's no signer, no Jupiter swap instruction builder, no real `simulateTransaction` call.

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
http_url = "https://api.mainnet-beta.solana.com"
ws_url = "wss://api.mainnet-beta.solana.com"
commitment = "processed"
```

For better latency on Pump.fun trading, point `ws_url` at a dedicated provider like `wss://pump.helius-rpc.com/` (Helius's public Pump.fun endpoint, no key required). For other DEXes, use `wss://mainnet.helius-rpc.com/?api-key=YOUR_KEY` or your own RPC.

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
- The scaffold has no wallet signer loaded, so even in non-dry-run mode, the bot cannot broadcast a trade yet. Do not add a funded key until the TODO list is closed.
