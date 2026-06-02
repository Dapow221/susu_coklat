export type EventKind =
  | "info"
  | "warn"
  | "error"
  | "heartbeat"
  | "dry_run_opportunity"
  | "dry_run_executed";

export type EventLevel = "info" | "warn" | "error";

export interface EngineEvent {
  id: string;
  seq: number;
  ts: string;
  level: EventLevel;
  kind: EventKind;
  message: string;
}

export interface EventsResponse {
  events: EngineEvent[];
  next_cursor: number;
  dry_run: boolean;
}

export class EngineApi {
  constructor(private readonly baseUrl: string) {}

  async status(): Promise<string> {
    try {
      const response = await fetch(`${this.baseUrl}/status`);
      if (!response.ok) {
        return "Engine API is not reachable yet.";
      }
      return response.text();
    } catch {
      return "Engine API is not reachable yet.";
    }
  }

  async events(since: number): Promise<EventsResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/events?since=${since}`);
      if (!response.ok) {
        return { events: [], next_cursor: since, dry_run: true };
      }
      return (await response.json()) as EventsResponse;
    } catch {
      return { events: [], next_cursor: since, dry_run: true };
    }
  }
}
