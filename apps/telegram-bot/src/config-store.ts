import { readFile, writeFile } from "node:fs/promises";

export class BotConfigStore {
  constructor(private readonly path: string) {}

  async setRiskNumber(key: string, value: number): Promise<void> {
    await this.replaceRiskValue(key, String(value));
  }

  async setRiskBoolean(key: string, value: boolean): Promise<void> {
    await this.replaceRiskValue(key, String(value));
  }

  private async replaceRiskValue(key: string, value: string): Promise<void> {
    const raw = await readFile(this.path, "utf8");
    const pattern = new RegExp(`^${key}\\s*=\\s*.*$`, "m");
    if (!pattern.test(raw)) {
      throw new Error(`Config key not found: ${key}`);
    }
    await writeFile(this.path, raw.replace(pattern, `${key} = ${value}`));
  }
}

