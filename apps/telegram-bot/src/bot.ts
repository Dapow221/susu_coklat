import { Telegraf } from "telegraf";
import { EngineApi } from "./api-client.js";
import { BotConfigStore } from "./config-store.js";
import { registerCommands } from "./commands.js";
import { startAlertPoller } from "./alerts.js";

export function createBot(token: string): Telegraf {
  const bot = new Telegraf(token);
  const engine = new EngineApi(process.env.ENGINE_API_URL ?? "http://127.0.0.1:8787");
  const config = new BotConfigStore("config/bot.toml");
  const adminChatId = process.env.TELEGRAM_ADMIN_CHAT_ID
    ? Number(process.env.TELEGRAM_ADMIN_CHAT_ID)
    : undefined;

  registerCommands(bot, engine, config);

  // Run the alert poller in the background so the bot can surface engine events
  // to the admin chat without manual commands.
  startAlertPoller({ engine, bot, adminChatId, pollIntervalMs: 2000 });

  return bot;
}

