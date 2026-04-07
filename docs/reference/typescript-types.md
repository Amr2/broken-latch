# TypeScript Types

Full type definitions for the broken-latch JavaScript SDK.

## Core Types

```typescript
export type GamePhase =
  | "NotRunning"
  | "Launching"
  | "InLobby"
  | "ChampSelect"
  | "Loading"
  | "InGame"
  | "EndGame";

export interface PhaseChangeEvent {
  previous: GamePhase;
  current: GamePhase;
  timestamp: number; // Unix ms
}

export interface GameSession {
  allyTeam: SessionPlayer[];
  enemyTeam: SessionPlayer[];
  localPlayer: SessionPlayer;
  gameMode: string;
  mapId: number;
}

export interface SessionPlayer {
  summonerName: string;
  puuid: string;
  championId: number;
  championName: string;
  teamId: number;
  position?: "TOP" | "JUNGLE" | "MID" | "BOTTOM" | "UTILITY";
}
```

## Widget Types

```typescript
export interface WidgetConfig {
  id: string;
  url: string;
  defaultPosition: { x: number; y: number };
  defaultSize: { width: number; height: number };
  minSize?: { width: number; height: number };
  maxSize?: { width: number; height: number };
  draggable?: boolean;
  resizable?: boolean;
  persistPosition?: boolean;
  clickThrough?: boolean;
  opacity?: number;
  showInPhases?: GamePhase[];
}

export interface WidgetState {
  id: string;
  visible: boolean;
  position: { x: number; y: number };
  size: { width: number; height: number };
  opacity: number;
  clickThrough: boolean;
}
```

## Notification Types

```typescript
export type NotificationType = "info" | "success" | "warning" | "error";

export interface NotificationOptions {
  duration?: number;
  type?: NotificationType;
  title?: string;
}
```

## SDK Interface

```typescript
export interface LOLOverlaySDK {
  init(config: { appId: string; version: string }): void;

  game: {
    onPhaseChange(callback: (event: PhaseChangeEvent) => void): void;
    getPhase(): Promise<GamePhase>;
    getSession(): Promise<GameSession | null>;
  };

  windows: {
    create(config: WidgetConfig): Promise<string>;
    show(widgetId: string): Promise<void>;
    hide(widgetId: string): Promise<void>;
    toggle(widgetId: string): Promise<void>;
    setOpacity(widgetId: string, opacity: number): Promise<void>;
    setPosition(widgetId: string, position: { x: number; y: number }): Promise<void>;
    setClickThrough(widgetId: string, enabled: boolean): Promise<void>;
    destroy(widgetId: string): Promise<void>;
  };

  hotkeys: {
    register(id: string, keys: string, callback: () => void): Promise<void>;
    unregister(id: string): Promise<void>;
    isRegistered(keys: string): Promise<boolean>;
  };

  storage: {
    get(key: string): Promise<unknown | null>;
    set(key: string, value: unknown): Promise<void>;
    delete(key: string): Promise<void>;
    clear(): Promise<void>;
  };

  notifications: {
    show(message: string, options?: NotificationOptions): Promise<void>;
  };

  messaging: {
    send(target: string, event: string, data?: unknown): Promise<void>;
    on(event: string, callback: (data: unknown) => void): void;
    off(event: string, callback?: (data: unknown) => void): void;
  };

  platform: {
    getVersion(): Promise<string>;
    getAppId(): string | null;
    SDK_VERSION: string;
    API_BASE: string;
  };
}

declare global {
  interface Window {
    LOLOverlay: LOLOverlaySDK;
  }
}
```

## Usage with TypeScript

If you're building with a bundler (Vite, webpack), you can import types from your project's type declarations:

```typescript
// types.d.ts
/// <reference path="./path/to/sdk-types.d.ts" />

// app.ts
const phase: GamePhase = await LOLOverlay.game.getPhase();
```
