import type { Telegraf } from "telegraf";
import type { EngineApi } from "./api-client.js";
import type { BotConfigStore } from "./config-store.js";

function isAdmin(chatId: number): boolean {
  const admin = process.env.TELEGRAM_ADMIN_CHAT_ID;
  return !admin || admin === String(chatId);
}

export function registerCommands(bot: Telegraf, engine: EngineApi, config: BotConfigStore): void {
  bot.start((ctx) => {
    if (!isAdmin(ctx.chat.id)) return;
    ctx.reply(
      "Bot online. Running in background mode. Commands:\n" +
        "/status - check engine state\n" +
        "/set_dry_run on|off - toggle dry-run mode"
    );
  });

  bot.command("status", async (ctx) => {
    if (!isAdmin(ctx.chat.id)) return;
    const status = await engine.status();
    ctx.reply(status);
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
