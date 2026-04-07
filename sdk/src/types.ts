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

/** Minimal SDK interface used by sub-modules to avoid circular imports. */
export interface ISDK {
  request(
    method: string,
    endpoint: string,
    data?: unknown,
  ): Promise<Record<string, unknown>>;
  getAppId(): string | null;
}
