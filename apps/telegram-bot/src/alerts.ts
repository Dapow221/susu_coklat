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
        console.log(`[poller] initialized, cursor=${cursor}, current next_cursor=${next_cursor}`);
        return;
      }

      if (events.length === 0) {
        return;
      }

      console.log(`[poller] fetched ${events.length} new events, cursor=${cursor}, next=${next_cursor}`);
      for (const event of events) {
        try {
          const result = await bot.telegram.sendMessage(adminChatId, formatEvent(event));
          console.log(`[poller] alert delivered: seq=${event.seq} kind=${event.kind} msg_id=${result.message_id}`);
        } catch (err: unknown) {
          const msg = err instanceof Error ? err.message : String(err);
          const code = (err as { code?: number }).code;
          console.error(`[poller] send failed: code=${code} msg=${msg}`);
        }
      }
      cursor = next_cursor;
      consecutiveErrors = 0;
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      consecutiveErrors += 1;
      if (consecutiveErrors === 1 || consecutiveErrors % 5 === 0) {
        console.error(`[poller] tick error (consecutive=${consecutiveErrors}): ${msg}`);
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
