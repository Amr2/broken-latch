import type { ISDK, GamePhase, PhaseChangeEvent, GameSession } from "./types";

/**
 * Game lifecycle API — listen for phase changes and query game state.
 */
export class Game {
  private sdk: ISDK;
  private listeners: Map<string, ((event: PhaseChangeEvent) => void)[]> =
    new Map();

  constructor(sdk: ISDK) {
    this.sdk = sdk;
    this.setupEventListeners();
  }

  /**
   * Listen for game phase changes.
   * @param callback - Function called whenever the phase transitions.
   */
  public onPhaseChange(callback: (event: PhaseChangeEvent) => void): void {
    if (!this.listeners.has("phase_change")) {
      this.listeners.set("phase_change", []);
    }
    this.listeners.get("phase_change")!.push(callback);
  }

  /**
   * Get the current game phase.
   */
  public async getPhase(): Promise<GamePhase> {
    const response = await this.sdk.request("GET", "/game/phase");
    return response["phase"] as GamePhase;
  }

  /**
   * Get current game session data (available from ChampSelect onwards).
   */
  public async getSession(): Promise<GameSession> {
    const response = await this.sdk.request("GET", "/game/session");
    return response as unknown as GameSession;
  }

  private setupEventListeners(): void {
    window.addEventListener("broken-latch:game_phase_changed", ((
      event: CustomEvent,
    ) => {
      const listeners = this.listeners.get("phase_change") ?? [];
      listeners.forEach((cb) => cb(event.detail as PhaseChangeEvent));
    }) as EventListener);
  }
}
