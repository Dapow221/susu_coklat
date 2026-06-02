import type { Telegraf } from "telegraf";
import type { EngineApi, EngineEvent } from "./api-client.js";

export type AlertLevel = "info" | "warn" | "error";

export interface Alert {
  level: AlertLevel;
  message: string;
}

const KIND_PREFIX: Record<string, string> = {
  info: "[i]",
  warn: "[!]",
  error: "[X]",
  heartbeat: "[HB]",
  dry_run_opportunity: "[OPP]",
  dry_run_executed: "[EXE]",
};

export function formatEvent(event: EngineEvent): string {
  const prefix = KIND_PREFIX[event.kind] ?? "[?]";
  const iso = new Date(event.ts).toISOString();
  const date = iso.substring(0, 10); // YYYY-MM-DD
  const time = iso.substring(11, 19); // HH:MM:SS
  return `${prefix} ${event.kind.toUpperCase()} @ ${date} ${time} UTC\n${event.message}`;
}

interface PollerOptions {
  engine: EngineApi;
  bot: Telegraf;
  adminChatId?: number;
  pollIntervalMs?: number;
}

export function startAlertPoller(opts: PollerOptions): void {
  const { engine, bot, adminChatId, pollIntervalMs = 2000 } = opts;

  if (!adminChatId) {
    console.warn("alert poller: TELEGRAM_ADMIN_CHAT_ID not set; alerts will not be delivered");
    return;
  }

  let cursor = 0;
  let consecutiveErrors = 0;
  let initialized = false;

  const tick = async (): Promise<void> => {
    try {
      const { events, next_cursor } = await engine.events(cursor);
      if (!initialized) {
        cursor = next_cursor;
        initialized = true;
        consecutiveErrors = 0;
        return;
      }

      for (const event of events) {
        try {
          await bot.telegram.sendMessage(adminChatId, formatEvent(event));
          console.log(`alert delivered: seq=${event.seq} kind=${event.kind}`);
        } catch (err) {
          console.error("alert send failed", err);
        }
      }
      if (next_cursor > cursor) cursor = next_cursor;
      consecutiveErrors = 0;
    } catch (err) {
      consecutiveErrors += 1;
      if (consecutiveErrors === 1 || consecutiveErrors % 10 === 0) {
        console.error(`alert poller error (consecutive=${consecutiveErrors})`, err);
      }
    } finally {
      const backoff = Math.min(
        pollIntervalMs * Math.pow(1.5, Math.min(consecutiveErrors, 6)),
        30000
      );
      setTimeout(tick, backoff);
    }
  };

  setTimeout(tick, 1000);
  console.log(`alert poller started; delivering to chat ${adminChatId}`);
}
