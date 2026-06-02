import { Telegraf } from "telegraf";
import { EngineApi } from "./api-client.js";
import { BotConfigStore } from "./config-store.js";
import { registerCommands } from "./commands.js";

export function createBot(token: string): Telegraf {
  const bot = new Telegraf(token);
  const engine = new EngineApi(process.env.ENGINE_API_URL ?? "http://127.0.0.1:8787");
  const config = new BotConfigStore("config/bot.toml");

  registerCommands(bot, engine, config);
  return bot;
}

