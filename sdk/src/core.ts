import { Game } from "./game";
import { Windows } from "./windows";
import { Hotkeys } from "./hotkeys";
import { Storage } from "./storage";
import { Notifications } from "./notifications";
import { Messaging } from "./messaging";
import { Platform } from "./platform";
import type { InitConfig, ISDK } from "./types";

/**
 * LOLOverlay SDK — main entry point for broken-latch platform apps.
 * Access via the global `LOLOverlay` object after loading the script.
 */
class LOLOverlaySDK implements ISDK {
  private appId: string | null = null;
  private appVersion: string | null = null;
  private readonly apiBaseUrl = "http://localhost:45678/api";
  private authToken: string | null = null;

  /** Game lifecycle API */
  public readonly game: Game;
  /** Widget window API */
  public readonly windows: Windows;
  /** Global hotkey API */
  public readonly hotkeys: Hotkeys;
  /** Persistent storage API */
  public readonly storage: Storage;
  /** Toast notification API */
  public readonly notify: Notifications;
  /** Inter-app messaging API */
  public readonly messaging: Messaging;
  /** Platform info API */
  public readonly platform: Platform;

  constructor() {
    this.game = new Game(this);
    this.windows = new Windows(this);
    this.hotkeys = new Hotkeys(this);
    this.storage = new Storage(this);
    this.notify = new Notifications(this);
    this.messaging = new Messaging(this);
    this.platform = new Platform(this);
  }

  /**
   * Initialize the SDK with your app credentials.
   * Must be called before using any other API.
   *
   * @example
   * await LOLOverlay.init({ appId: "my-app", version: "1.0.0" });
   */
  public async init(config: InitConfig): Promise<void> {
    this.appId = config.appId;
    this.appVersion = config.version;

    // Auth token is injected by the platform as a <meta> tag
    const tokenMeta = document.querySelector(
      'meta[name="broken-latch-token"]',
    );
    if (tokenMeta) {
      this.authToken = tokenMeta.getAttribute("content");
    }

    if (!this.authToken) {
      throw new Error(
        "[LOLOverlay SDK] Failed to retrieve app authentication token. " +
          "Ensure the app is loaded through the broken-latch platform.",
      );
    }

    console.log(
      `[LOLOverlay SDK] Initialized for app: ${this.appId} v${this.appVersion}`,
    );
  }

  /**
   * Internal: make an authenticated HTTP request to the platform API.
   * @internal
   */
  public async request(
    method: string,
    endpoint: string,
    data?: unknown,
  ): Promise<Record<string, unknown>> {
    if (!this.authToken) {
      throw new Error(
        "[LOLOverlay SDK] SDK not initialized. Call init() first.",
      );
    }

    const url = `${this.apiBaseUrl}${endpoint}`;

    const options: RequestInit = {
      method,
      headers: {
        "Content-Type": "application/json",
        "X-broken-latch-App-Token": this.authToken,
      },
    };

    if (data !== undefined && (method === "POST" || method === "PUT")) {
      options.body = JSON.stringify(data);
    }

    const response = await fetch(url, options);

    if (!response.ok) {
      throw new Error(
        `[LOLOverlay SDK] API error ${response.status}: ${response.statusText}`,
      );
    }

    return response.json() as Promise<Record<string, unknown>>;
  }

  /**
   * Get the current app ID (set during init).
   */
  public getAppId(): string | null {
    return this.appId;
  }

  /**
   * Get the current app version (set during init).
   */
  public getVersion(): string | null {
    return this.appVersion;
  }
}

const sdkInstance = new LOLOverlaySDK();

// Attach to window global for script-tag usage
if (typeof window !== "undefined") {
  (window as unknown as Record<string, unknown>)["LOLOverlay"] = sdkInstance;
}

export default sdkInstance;
