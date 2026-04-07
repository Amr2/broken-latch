import type { ISDK } from "./types";

/**
 * Inter-app messaging API — send and receive messages between platform apps.
 */
export class Messaging {
  private sdk: ISDK;
  private listeners: ((sender: string, message: unknown) => void)[] = [];

  constructor(sdk: ISDK) {
    this.sdk = sdk;
    this.setupMessageListener();
  }

  /**
   * Send a message to another app.
   */
  public async send(targetAppId: string, message: unknown): Promise<void> {
    await this.sdk.request("POST", "/messaging/send", {
      fromAppId: this.sdk.getAppId(),
      toAppId: targetAppId,
      message,
    });
  }

  /**
   * Listen for messages from other apps.
   * @param callback - Receives sender app ID and the message payload.
   */
  public on(callback: (sender: string, message: unknown) => void): void {
    this.listeners.push(callback);
  }

  private setupMessageListener(): void {
    window.addEventListener("broken-latch:app_message", ((
      event: CustomEvent,
    ) => {
      const { sender, message } = event.detail as {
        sender: string;
        message: unknown;
      };
      this.listeners.forEach((cb) => cb(sender, message));
    }) as EventListener);
  }
}
