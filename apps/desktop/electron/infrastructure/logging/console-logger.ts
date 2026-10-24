import type { Logger } from "../../application/interfaces/logger-interfaces";

export class ConsoleLogger implements Logger {
  error(message: string): void {
    console.error(message);
  }
}
