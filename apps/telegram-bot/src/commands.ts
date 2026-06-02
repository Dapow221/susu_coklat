import type { Telegraf } from "telegraf";
import type { EngineApi } from "./api-client.js";
import type { BotConfigStore } from "./config-store.js";

function isAdmin(chatId: number): boolean {
  const admin = process.env.TELEGRAM_ADMIN_CHAT_ID;
  return !admin || admin === String(chatId);
}

export function registerCommands(bot: Telegraf, engine: EngineApi, config: BotConfigStore): void {
  bot.start((ctx) => {
    ctx.reply("Solana MEV bot control online. Default mode should stay dry_run until tested.");
  });

  bot.command("status", async (ctx) => {
    if (!isAdmin(ctx.chat.id)) return;
    const status = await engine.status();
    ctx.reply(status);
  });

  bot.command("pause", async (ctx) => {
    if (!isAdmin(ctx.chat.id)) return;
    await engine.pause();
    ctx.reply("Engine paused.");
  });

  bot.command("resume", async (ctx) => {
    if (!isAdmin(ctx.chat.id)) return;
    await engine.resume();
    ctx.reply("Engine resumed.");
  });

  bot.command("set_threshold", async (ctx) => {
    if (!isAdmin(ctx.chat.id)) return;
    const value = Number(ctx.message.text.split(/\s+/)[1]);
    if (!Number.isFinite(value) || value < 0) {
      ctx.reply("Usage: /set_threshold <bps>");
      return;
    }
    await config.setRiskNumber("min_profit_bps", value);
    ctx.reply(`min_profit_bps set to ${value}. Restart engine to apply file config.`);
  });

  bot.command("set_dry_run", async (ctx) => {
    if (!isAdmin(ctx.chat.id)) return;
    const value = ctx.message.text.split(/\s+/)[1];
    if (value !== "on" && value !== "off") {
      ctx.reply("Usage: /set_dry_run on|off");
      return;
    }
    await config.setRiskBoolean("dry_run", value === "on");
    ctx.reply(`dry_run set to ${value}. Restart engine to apply file config.`);
  });
}

