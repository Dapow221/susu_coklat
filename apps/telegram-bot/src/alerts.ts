export type AlertLevel = "info" | "warn" | "error";

export interface Alert {
  level: AlertLevel;
  message: string;
}

