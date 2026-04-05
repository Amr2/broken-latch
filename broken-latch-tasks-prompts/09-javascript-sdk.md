# Task 09: JavaScript SDK for App Developers

**Platform: broken-latch**  
**Dependencies:** Task 01, 04, 05, 06, 08  
**Estimated Complexity:** Medium-High  
**Priority:** P0 (Critical - apps depend on this)

---

## Objective

Build the JavaScript SDK (`loloverlay.js`) that app developers load to integrate with the broken-latch platform. This SDK wraps all platform APIs into a clean, documented JavaScript interface. It's served at `http://localhost:45678/sdk/loloverlay.js` and provides the `LOLOverlay` global object.

**Note:** Even though the platform is now called "broken-latch", we keep the SDK namespace as `LOLOverlay` for app developer familiarity (as documented in the original spec).

---

## Context

The SDK is the primary way apps interact with the platform. It must:

- Be lightweight (<50KB minified)
- Work in WebView2 contexts (ES2020 compatible)
- Provide TypeScript definitions
- Handle authentication automatically (using app token from manifest)
- Abstract away HTTP API and Tauri event details
- Be fully documented with JSDoc

This is the developer experience layer - if this is confusing or buggy, no one will build apps.

---

## What You Need to Build

### 1. SDK Structure (`sdk/loloverlay.js`)

Create `sdk/` directory in project root:

```
sdk/
├── loloverlay.js           # Main SDK file (bundled, will be served)
├── src/
│   ├── core.ts            # Core SDK class
│   ├── game.ts            # Game lifecycle API
│   ├── windows.ts         # Window management API
│   ├── hotkeys.ts         # Hotkey API
│   ├── storage.ts         # Storage API
│   ├── notifications.ts   # Toast notification API
│   ├── messaging.ts       # Inter-app messaging API
│   ├── platform.ts        # Platform info API
│   └── types.ts           # TypeScript definitions
├── package.json
├── tsconfig.json
└── rollup.config.js       # Bundle to single file
```

### 2. SDK Package Setup

**`sdk/package.json`:**

```json
{
  "name": "@broken-latch/sdk",
  "version": "1.0.0",
  "description": "JavaScript SDK for broken-latch platform apps",
  "main": "dist/loloverlay.js",
  "types": "dist/loloverlay.d.ts",
  "scripts": {
    "build": "rollup -c",
    "dev": "rollup -c -w",
    "typecheck": "tsc --noEmit"
  },
  "devDependencies": {
    "@rollup/plugin-typescript": "^11.0.0",
    "@rollup/plugin-terser": "^0.4.0",
    "rollup": "^4.0.0",
    "typescript": "^5.0.0"
  }
}
```

**`sdk/tsconfig.json`:**

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ES2020",
    "lib": ["ES2020", "DOM"],
    "declaration": true,
    "declarationDir": "./dist",
    "outDir": "./dist",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src/**/*"]
}
```

**`sdk/rollup.config.js`:**

```javascript
import typescript from "@rollup/plugin-typescript";
import terser from "@rollup/plugin-terser";

export default {
  input: "src/core.ts",
  output: {
    file: "dist/loloverlay.js",
    format: "iife",
    name: "LOLOverlay",
    sourcemap: true,
  },
  plugins: [
    typescript(),
    terser(), // Minify for production
  ],
};
```

### 3. Type Definitions (`sdk/src/types.ts`)

```typescript
export type GamePhase =
  | "NotRunning"
  | "InLobby"
  | "ChampSelect"
  | "Loading"
  | "InGame"
  | "EndGame";

export interface PhaseChangeEvent {
  previous: GamePhase;
  current: GamePhase;
  timestamp: number;
}

export interface GameSession {
  allyTeam: SessionPlayer[];
  enemyTeam: SessionPlayer[];
  localPlayer: SessionPlayer;
}

export interface SessionPlayer {
  summonerName: string;
  puuid: string;
  championId: number;
  championName: string;
  summonerSpells: [number, number];
}

export interface WidgetConfig {
  id: string;
  title: string;
  width: number;
  height: number;
  x?: number;
  y?: number;
  draggable: boolean;
  resizable: boolean;
  persistPosition: boolean;
  opacity?: number;
  clickThrough?: boolean;
  showInPhases?: GamePhase[];
}

export interface WidgetState {
  visible: boolean;
  position: { x: number; y: number };
  size: { width: number; height: number };
  opacity: number;
  clickThrough: boolean;
}

export interface HotkeyConfig {
  id: string;
  keys: string;
  description?: string;
  onPress: () => void;
}

export interface ToastConfig {
  message: string;
  type?: "info" | "success" | "warning" | "error";
  duration?: number;
  position?: "top-left" | "top-right" | "bottom-left" | "bottom-right";
}

export interface PlatformInfo {
  version: string;
  installedApps: AppInfo[];
  currentApp: string;
}

export interface AppInfo {
  id: string;
  name: string;
  version: string;
  enabled: boolean;
}

export interface InitConfig {
  appId: string;
  version: string;
}
```

### 4. Core SDK (`sdk/src/core.ts`)

```typescript
import { Game } from "./game";
import { Windows } from "./windows";
import { Hotkeys } from "./hotkeys";
import { Storage } from "./storage";
import { Notifications } from "./notifications";
import { Messaging } from "./messaging";
import { Platform } from "./platform";
import type { InitConfig } from "./types";

class LOLOverlaySDK {
  private appId: string | null = null;
  private appVersion: string | null = null;
  private apiBaseUrl = "http://localhost:45678/api";
  private authToken: string | null = null;

  public game: Game;
  public windows: Windows;
  public hotkeys: Hotkeys;
  public storage: Storage;
  public notify: Notifications;
  public messaging: Messaging;
  public platform: Platform;

  constructor() {
    // Initialize API modules
    this.game = new Game(this);
    this.windows = new Windows(this);
    this.hotkeys = new Hotkeys(this);
    this.storage = new Storage(this);
    this.notify = new Notifications(this);
    this.messaging = new Messaging(this);
    this.platform = new Platform(this);
  }

  /**
   * Initialize the SDK with app credentials
   * Must be called before using any other API
   */
  public async init(config: InitConfig): Promise<void> {
    this.appId = config.appId;
    this.appVersion = config.version;

    // Fetch app token from platform (injected in manifest during app load)
    // In real implementation, this would be provided by the platform
    const tokenMeta = document.querySelector('meta[name="broken-latch-token"]');
    if (tokenMeta) {
      this.authToken = tokenMeta.getAttribute("content");
    }

    if (!this.authToken) {
      throw new Error(
        "[LOLOverlay SDK] Failed to retrieve app authentication token",
      );
    }

    console.log(
      `[LOLOverlay SDK] Initialized for app: ${this.appId} v${this.appVersion}`,
    );
  }

  /**
   * Internal: Make authenticated HTTP request to platform API
   */
  public async request(
    method: string,
    endpoint: string,
    data?: any,
  ): Promise<any> {
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

    if (data && (method === "POST" || method === "PUT")) {
      options.body = JSON.stringify(data);
    }

    const response = await fetch(url, options);

    if (!response.ok) {
      throw new Error(`[LOLOverlay SDK] API error: ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Get current app ID
   */
  public getAppId(): string | null {
    return this.appId;
  }
}

// Export singleton instance as global
const sdkInstance = new LOLOverlaySDK();

// Attach to window for app access
if (typeof window !== "undefined") {
  (window as any).LOLOverlay = sdkInstance;
}

export default sdkInstance;
```

### 5. Game Lifecycle API (`sdk/src/game.ts`)

```typescript
import type LOLOverlaySDK from "./core";
import type { GamePhase, PhaseChangeEvent, GameSession } from "./types";

export class Game {
  private sdk: typeof LOLOverlaySDK;
  private listeners: Map<string, ((event: PhaseChangeEvent) => void)[]> =
    new Map();

  constructor(sdk: typeof LOLOverlaySDK) {
    this.sdk = sdk;
    this.setupEventListeners();
  }

  /**
   * Listen for game phase changes
   * @param callback - Function called when phase changes
   */
  public onPhaseChange(callback: (event: PhaseChangeEvent) => void): void {
    if (!this.listeners.has("phase_change")) {
      this.listeners.set("phase_change", []);
    }
    this.listeners.get("phase_change")!.push(callback);
  }

  /**
   * Get current game phase
   */
  public async getPhase(): Promise<GamePhase> {
    const response = await this.sdk.request("GET", "/game/phase");
    return response.phase;
  }

  /**
   * Get current game session data (available from ChampSelect onwards)
   */
  public async getSession(): Promise<GameSession> {
    const response = await this.sdk.request("GET", "/game/session");
    return response;
  }

  private setupEventListeners(): void {
    // Listen for game_phase_changed events from platform
    // Platform pushes these via webhook or WebSocket
    window.addEventListener("broken-latch:game_phase_changed", ((
      event: CustomEvent,
    ) => {
      const listeners = this.listeners.get("phase_change") || [];
      listeners.forEach((callback) => callback(event.detail));
    }) as EventListener);
  }
}
```

### 6. Window Management API (`sdk/src/windows.ts`)

```typescript
import type LOLOverlaySDK from "./core";
import type { WidgetConfig, WidgetState } from "./types";

export class Windows {
  private sdk: typeof LOLOverlaySDK;

  constructor(sdk: typeof LOLOverlaySDK) {
    this.sdk = sdk;
  }

  /**
   * Create a new widget window (defined in manifest, this activates it)
   */
  public async create(config: WidgetConfig): Promise<void> {
    await this.sdk.request("POST", "/windows/create", {
      appId: this.sdk.getAppId(),
      ...config,
    });
  }

  /**
   * Show a widget window
   */
  public async show(windowId: string): Promise<void> {
    await this.sdk.request("POST", "/windows/show", {
      appId: this.sdk.getAppId(),
      windowId,
    });
  }

  /**
   * Hide a widget window
   */
  public async hide(windowId: string): Promise<void> {
    await this.sdk.request("POST", "/windows/hide", {
      appId: this.sdk.getAppId(),
      windowId,
    });
  }

  /**
   * Set widget opacity (0.0 - 1.0)
   */
  public async setOpacity(windowId: string, opacity: number): Promise<void> {
    await this.sdk.request("POST", "/windows/set-opacity", {
      appId: this.sdk.getAppId(),
      windowId,
      opacity: Math.max(0, Math.min(1, opacity)),
    });
  }

  /**
   * Toggle click-through mode for a widget
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
   * Get current state of a widget
   */
  public async getState(windowId: string): Promise<WidgetState> {
    const response = await this.sdk.request(
      "GET",
      `/windows/${windowId}/state?appId=${this.sdk.getAppId()}`,
    );
    return response;
  }
}
```

### 7. Hotkey API (`sdk/src/hotkeys.ts`)

```typescript
import type LOLOverlaySDK from "./core";
import type { HotkeyConfig } from "./types";

export class Hotkeys {
  private sdk: typeof LOLOverlaySDK;
  private callbacks: Map<string, () => void> = new Map();

  constructor(sdk: typeof LOLOverlaySDK) {
    this.sdk = sdk;
    this.setupHotkeyListener();
  }

  /**
   * Register a global hotkey
   */
  public async register(config: HotkeyConfig): Promise<void> {
    // Store callback
    this.callbacks.set(config.id, config.onPress);

    // Register with platform
    await this.sdk.request("POST", "/hotkeys/register", {
      appId: this.sdk.getAppId(),
      hotkeyId: config.id,
      keys: config.keys,
      description: config.description || "",
    });
  }

  /**
   * Unregister a hotkey
   */
  public async unregister(hotkeyId: string): Promise<void> {
    this.callbacks.delete(hotkeyId);

    await this.sdk.request("POST", "/hotkeys/unregister", {
      appId: this.sdk.getAppId(),
      hotkeyId,
    });
  }

  /**
   * Check if a key combination is already registered
   */
  public async isRegistered(keys: string): Promise<boolean> {
    const response = await this.sdk.request(
      "GET",
      `/hotkeys/check?keys=${encodeURIComponent(keys)}`,
    );
    return response.registered;
  }

  private setupHotkeyListener(): void {
    // Listen for hotkey press events from platform
    window.addEventListener("broken-latch:hotkey_pressed", ((
      event: CustomEvent,
    ) => {
      const { hotkeyId } = event.detail;
      const callback = this.callbacks.get(hotkeyId);
      if (callback) {
        callback();
      }
    }) as EventListener);
  }
}
```

### 8. Storage API (`sdk/src/storage.ts`)

```typescript
import type LOLOverlaySDK from "./core";

export class Storage {
  private sdk: typeof LOLOverlaySDK;

  constructor(sdk: typeof LOLOverlaySDK) {
    this.sdk = sdk;
  }

  /**
   * Store a value (automatically scoped to your app)
   */
  public async set(key: string, value: any): Promise<void> {
    await this.sdk.request("POST", `/storage/${this.sdk.getAppId()}/${key}`, {
      value,
    });
  }

  /**
   * Retrieve a value
   */
  public async get<T = any>(key: string): Promise<T | null> {
    try {
      const response = await this.sdk.request(
        "GET",
        `/storage/${this.sdk.getAppId()}/${key}`,
      );
      return response.value;
    } catch {
      return null;
    }
  }

  /**
   * Delete a value
   */
  public async delete(key: string): Promise<void> {
    await this.sdk.request("DELETE", `/storage/${this.sdk.getAppId()}/${key}`);
  }

  /**
   * Clear all storage for this app
   */
  public async clear(): Promise<void> {
    await this.sdk.request("DELETE", `/storage/${this.sdk.getAppId()}`);
  }
}
```

### 9. Notifications API (`sdk/src/notifications.ts`)

```typescript
import type LOLOverlaySDK from "./core";
import type { ToastConfig } from "./types";

export class Notifications {
  private sdk: typeof LOLOverlaySDK;

  constructor(sdk: typeof LOLOverlaySDK) {
    this.sdk = sdk;
  }

  /**
   * Show a toast notification in the overlay
   */
  public toast(config: ToastConfig): void {
    this.sdk
      .request("POST", "/notify/toast", {
        appId: this.sdk.getAppId(),
        message: config.message,
        type: config.type || "info",
        duration: config.duration || 3000,
        position: config.position || "top-right",
      })
      .catch((err) => {
        console.error("[LOLOverlay SDK] Toast notification failed:", err);
      });
  }
}
```

### 10. Inter-App Messaging API (`sdk/src/messaging.ts`)

```typescript
import type LOLOverlaySDK from "./core";

export class Messaging {
  private sdk: typeof LOLOverlaySDK;
  private listeners: ((sender: string, message: any) => void)[] = [];

  constructor(sdk: typeof LOLOverlaySDK) {
    this.sdk = sdk;
    this.setupMessageListener();
  }

  /**
   * Send a message to another app
   */
  public async send(targetAppId: string, message: any): Promise<void> {
    await this.sdk.request("POST", "/messaging/send", {
      fromAppId: this.sdk.getAppId(),
      toAppId: targetAppId,
      message,
    });
  }

  /**
   * Listen for messages from other apps
   */
  public on(callback: (sender: string, message: any) => void): void {
    this.listeners.push(callback);
  }

  private setupMessageListener(): void {
    window.addEventListener("broken-latch:app_message", ((
      event: CustomEvent,
    ) => {
      const { sender, message } = event.detail;
      this.listeners.forEach((callback) => callback(sender, message));
    }) as EventListener);
  }
}
```

### 11. Platform Info API (`sdk/src/platform.ts`)

```typescript
import type LOLOverlaySDK from "./core";
import type { PlatformInfo } from "./types";

export class Platform {
  private sdk: typeof LOLOverlaySDK;

  constructor(sdk: typeof LOLOverlaySDK) {
    this.sdk = sdk;
  }

  /**
   * Get platform version and installed apps
   */
  public async getInfo(): Promise<PlatformInfo> {
    const response = await this.sdk.request("GET", "/platform/info");
    return response;
  }
}
```

---

## Integration with HTTP API Server (Task 08)

The SDK server in Task 08 will serve this file at `/sdk/loloverlay.js`:

```rust
// In src-tauri/src/sdk_server.rs
async fn serve_sdk() -> impl IntoResponse {
    let sdk_content = include_str!("../../sdk/dist/loloverlay.js");

    (
        StatusCode::OK,
        [
            ("Content-Type", "application/javascript"),
            ("Cache-Control", "public, max-age=3600"),
        ],
        sdk_content
    )
}
```

Apps load it via:

```html
<script src="http://localhost:45678/sdk/loloverlay.js"></script>
```

---

## Testing Requirements

### Unit Tests (sdk/src/**tests**)

```typescript
// game.test.ts
describe("Game API", () => {
  test("onPhaseChange registers listener", () => {
    const game = new Game(mockSDK);
    let called = false;
    game.onPhaseChange(() => {
      called = true;
    });

    // Trigger event
    window.dispatchEvent(
      new CustomEvent("broken-latch:game_phase_changed", {
        detail: {
          previous: "InLobby",
          current: "ChampSelect",
          timestamp: Date.now(),
        },
      }),
    );

    expect(called).toBe(true);
  });
});

// storage.test.ts
describe("Storage API", () => {
  test("set and get value", async () => {
    const storage = new Storage(mockSDK);
    await storage.set("test_key", { foo: "bar" });
    const value = await storage.get("test_key");
    expect(value.foo).toBe("bar");
  });
});
```

### Integration Tests

Create test app that uses SDK:

```html
<!-- test-app/index.html -->
<script src="http://localhost:45678/sdk/loloverlay.js"></script>
<script>
  LOLOverlay.init({ appId: "test-app", version: "1.0.0" }).then(() => {
    console.log("✅ SDK initialized");

    // Test game API
    LOLOverlay.game.onPhaseChange((event) => {
      console.log("Phase changed:", event);
    });

    // Test storage API
    LOLOverlay.storage.set("test", "value").then(() => {
      console.log("✅ Storage set");
    });
  });
</script>
```

### Manual Testing Checklist

- [ ] Build SDK: `cd sdk && npm run build`
- [ ] Verify `sdk/dist/loloverlay.js` exists and is <50KB minified
- [ ] Start platform with HTTP API server
- [ ] Load test app HTML in browser
- [ ] Verify SDK initializes without errors
- [ ] Test each API module (game, windows, hotkeys, storage, notifications)
- [ ] Verify TypeScript definitions work in IDE

---

## Acceptance Criteria

✅ **Complete when:**

1. SDK builds to single `loloverlay.js` file <50KB minified
2. TypeScript definitions (`loloverlay.d.ts`) are generated
3. All API modules (Game, Windows, Hotkeys, Storage, Notifications, Messaging, Platform) are implemented
4. SDK can be served via HTTP API at `/sdk/loloverlay.js`
5. Apps can load SDK and call `LOLOverlay.init()`
6. Authentication token is automatically included in all requests
7. All unit tests pass
8. Integration test app works end-to-end
9. JSDoc documentation is complete for all public methods

---

## Documentation Output

Generate `SDK_REFERENCE.md` with full API documentation:

````markdown
# LOLOverlay SDK Reference

## Installation

```html
<script src="http://localhost:45678/sdk/loloverlay.js"></script>
```
````

## Initialization

```javascript
await LOLOverlay.init({
  appId: "your-app-id",
  version: "1.0.0",
});
```

## API Reference

### LOLOverlay.game

**`onPhaseChange(callback)`**
Listen for game phase changes...

[Continue with full documentation]

```

---

## Dependencies for Next Tasks
- **Task 10** (Platform UI) will use this SDK for testing/demos
- **Task 12** (SDK Documentation) will reference this implementation
- **Task 13** (Developer CLI) will use SDK as template for new apps
- **Hunter Mode** will be the first real app using this SDK

---

## Files to Create/Modify

### New Files:
- `sdk/package.json`
- `sdk/tsconfig.json`
- `sdk/rollup.config.js`
- `sdk/src/core.ts`
- `sdk/src/game.ts`
- `sdk/src/windows.ts`
- `sdk/src/hotkeys.ts`
- `sdk/src/storage.ts`
- `sdk/src/notifications.ts`
- `sdk/src/messaging.ts`
- `sdk/src/platform.ts`
- `sdk/src/types.ts`
- `SDK_REFERENCE.md`

### Modified Files:
- `src-tauri/src/sdk_server.rs` (serve the built file)

---

## Expected Time: 8-10 hours
## Difficulty: Medium-High (API design + TypeScript)
```
