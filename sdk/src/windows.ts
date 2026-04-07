import type { ISDK, WidgetConfig, WidgetState } from "./types";

/**
 * Window management API — create and control widget windows.
 */
export class Windows {
  private sdk: ISDK;

  constructor(sdk: ISDK) {
    this.sdk = sdk;
  }

  /**
   * Create a new widget window (defined in manifest, this activates it).
   */
  public async create(config: WidgetConfig): Promise<void> {
    await this.sdk.request("POST", "/windows/create", {
      appId: this.sdk.getAppId(),
      ...config,
    });
  }

  /**
   * Show a widget window.
   */
  public async show(windowId: string): Promise<void> {
    await this.sdk.request("POST", "/windows/show", {
      appId: this.sdk.getAppId(),
      windowId,
    });
  }

  /**
   * Hide a widget window.
   */
  public async hide(windowId: string): Promise<void> {
    await this.sdk.request("POST", "/windows/hide", {
      appId: this.sdk.getAppId(),
      windowId,
    });
  }

  /**
   * Set widget opacity (0.0 – 1.0).
   */
  public async setOpacity(windowId: string, opacity: number): Promise<void> {
    await this.sdk.request("POST", "/windows/set-opacity", {
      appId: this.sdk.getAppId(),
      windowId,
      opacity: Math.max(0, Math.min(1, opacity)),
    });
  }

  /**
   * Toggle click-through mode for a widget.
   * When enabled, mouse events pass through the widget to whatever is below.
   */
  public async setClickThrough(
    windowId: string,
    enabled: boolean,
  ): Promise<void> {
    await this.sdk.request("POST", "/windows/set-click-through", {
      appId: this.sdk.getAppId(),
      windowId,
      clickThrough: enabled,
    });
  }

  /**
   * Get current state of a widget (position, size, visibility, opacity).
   */
  public async getState(windowId: string): Promise<WidgetState> {
    const response = await this.sdk.request(
      "GET",
      `/windows/${windowId}/state?appId=${this.sdk.getAppId()}`,
    );
    return response as unknown as WidgetState;
  }
}
