export class EngineApi {
  constructor(private readonly baseUrl: string) {}

  async status(): Promise<string> {
    const response = await fetch(`${this.baseUrl}/status`);
    if (!response.ok) {
      return "Engine API is not reachable yet.";
    }
    return response.text();
  }

  async pause(): Promise<void> {
    await fetch(`${this.baseUrl}/pause`, { method: "POST" });
  }

  async resume(): Promise<void> {
    await fetch(`${this.baseUrl}/resume`, { method: "POST" });
  }
}

