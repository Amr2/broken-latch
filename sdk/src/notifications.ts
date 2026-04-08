import type { ISDK, ToastConfig } from "./types";

/**
 * Notifications API — show toast messages in the overlay.
 */
export class Notifications {
  private sdk: ISDK;

  constructor(sdk: ISDK) {
    this.sdk = sdk;
  }

  /**
   * Show a toast notification.
   * Fire-and-forget — errors are logged to console, not thrown.
   */
  public toast(config: ToastConfig): void {
    this.sdk
      .request("POST", "/notify/toast", {
        appId: this.sdk.getAppId(),
        message: config.message,
        type: config.type ?? "info",
        duration: config.duration ?? 3000,
        position: config.position ?? "top-right",
      })
      .catch((err: unknown) => {
        console.error("[LOLOverlay SDK] Toast notification failed:", err);
      });
  }
}
