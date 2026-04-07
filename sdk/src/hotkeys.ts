import type { ISDK, HotkeyConfig } from "./types";

/**
 * Hotkey API — register and manage global keyboard shortcuts.
 */
export class Hotkeys {
  private sdk: ISDK;
  private callbacks: Map<string, () => void> = new Map();

  constructor(sdk: ISDK) {
    this.sdk = sdk;
    this.setupHotkeyListener();
  }

  /**
   * Register a global hotkey.
   * Keys format: "Ctrl+Shift+F1", "Alt+F", etc.
   */
  public async register(config: HotkeyConfig): Promise<void> {
    this.callbacks.set(config.id, config.onPress);

    await this.sdk.request("POST", "/hotkeys/register", {
      appId: this.sdk.getAppId(),
      hotkeyId: config.id,
      keys: config.keys,
      description: config.description ?? "",
    });
  }

  /**
   * Unregister a hotkey by ID.
   */
  public async unregister(hotkeyId: string): Promise<void> {
    this.callbacks.delete(hotkeyId);

    await this.sdk.request("POST", "/hotkeys/unregister", {
      appId: this.sdk.getAppId(),
      hotkeyId,
    });
  }

  /**
   * Check if a key combination is already registered by any app.
   */
  public async isRegistered(keys: string): Promise<boolean> {
    const response = await this.sdk.request(
      "GET",
      `/hotkeys/check?keys=${encodeURIComponent(keys)}`,
    );
    return response["registered"] as boolean;
  }

  private setupHotkeyListener(): void {
    window.addEventListener("broken-latch:hotkey_pressed", ((
      event: CustomEvent,
    ) => {
      const { hotkeyId } = event.detail as { hotkeyId: string };
      const callback = this.callbacks.get(hotkeyId);
      if (callback) {
        callback();
      }
    }) as EventListener);
  }
}
