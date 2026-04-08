/**
 * DevDashboard.tsx — broken-latch Platform Dev Simulator
 * =======================================================
 * This window is the primary development tool for the broken-latch platform.
 * It demonstrates every available platform capability and lets you simulate
 * game phase transitions to preview how overlay apps would behave in real use.
 *
 * ┌─────────────────────────────────────────────────────────────────────────┐
 * │  WHAT THIS DEMONSTRATES                                                 │
 * │                                                                         │
 * │  1. Game Phase Simulation  invoke('simulate_phase', { phase })          │
 * │     Triggers the phase-changed Tauri event → overlay swaps content      │
 * │                                                                         │
 * │  2. Overlay Visibility     invoke('show_overlay') / invoke('hide_overlay')│
 * │     Auto-managed per phase (hidden on NOT_RUNNING, shown otherwise)     │
 * │                                                                         │
 * │  3. Opacity Control        invoke('set_overlay_opacity', { opacity })   │
 * │     0.0 = invisible → 1.0 = fully opaque (via SetLayeredWindowAttributes)│
 * │                                                                         │
 * │  4. Screen Capture Toggle  invoke('set_screen_capture_visible', { visible })│
 * │     WDA_EXCLUDEFROMCAPTURE hides overlay from OBS/Discord streams       │
 * │                                                                         │
 * │  5. Interactive Regions    invoke('update_interactive_regions', { regions })│
 * │     Defines which Rect areas capture mouse input (SetWindowRgn)         │
 * │     Everything outside is click-through to the game                    │
 * │                                                                         │
 * │  6. Platform Config        invoke('get_platform_config')                │
 * │     Returns the TOML config loaded at startup from %APPDATA%           │
 * │                                                                         │
 * │  7. Live Event Log         listen('phase-changed', handler)             │
 * │     Shows all Tauri events in real-time as they fire                   │
 * └─────────────────────────────────────────────────────────────────────────┘
 *
 * ARCHITECTURE:
 *   This window  ←→  Rust backend (Tauri IPC)  ←→  Overlay window
 *   Both windows are separate WebviewWindows.  Communication happens via
 *   Tauri commands (invoke) and events (emit/listen).
 */

import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import './DevDashboard.css';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** The seven game phases the platform understands. */
type GamePhase =
  | 'NOT_RUNNING'
  | 'LAUNCHING'
  | 'IN_LOBBY'
  | 'CHAMP_SELECT'
  | 'LOADING'
  | 'IN_GAME'
  | 'END_GAME';

/** A timestamped line in the live event log. */
interface LogEntry {
  id: number;
  time: string;
  type: 'event' | 'command' | 'system' | 'error';
  message: string;
}

/** Shape of the config returned by `get_platform_config`. */
interface PlatformConfig {
  version: string;
  startup: { launch_with_windows: boolean; auto_update_check: boolean };
  overlay: { default_opacity: number; screen_capture_visible: boolean };
  performance: { performance_mode: boolean; cpu_usage_limit: number };
  developer: { debug_mode: boolean; show_console_logs: boolean; simulated_game_phases: boolean };
}

/** Shape of a Rect used in `update_interactive_regions`. */
interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

// ---------------------------------------------------------------------------
// Phase metadata — labels, icons, colours, and description for each phase
// ---------------------------------------------------------------------------

interface PhaseInfo {
  label: string;
  icon: string;
  color: string;
  description: string;
  overlayShown: boolean;
}

const PHASES: Record<GamePhase, PhaseInfo> = {
  NOT_RUNNING: {
    label: 'Not Running',
    icon: '⭕',
    color: '#555',
    description: 'Platform idle. No game process detected. Overlay is hidden.',
    overlayShown: false,
  },
  LAUNCHING: {
    label: 'Launching',
    icon: '🚀',
    color: '#f59e0b',
    description: 'League of Legends process detected, loading. Overlay shows startup banner.',
    overlayShown: true,
  },
  IN_LOBBY: {
    label: 'In Lobby',
    icon: '🏠',
    color: '#6366f1',
    description: 'Main client lobby. Overlay shows quick stats and match history.',
    overlayShown: true,
  },
  CHAMP_SELECT: {
    label: 'Champ Select',
    icon: '⚔️',
    color: '#8b5cf6',
    description: 'Champion selection phase. Overlay shows pick/ban tracker and team compositions.',
    overlayShown: true,
  },
  LOADING: {
    label: 'Loading',
    icon: '⏳',
    color: '#3b82f6',
    description: 'Game loading screen. Overlay shows champion lore and loading tips.',
    overlayShown: true,
  },
  IN_GAME: {
    label: 'In Game',
    icon: '🎮',
    color: '#22c55e',
    description: 'Active match. Overlay shows KDA, CS, gold, and game timer widgets.',
    overlayShown: true,
  },
  END_GAME: {
    label: 'End Game',
    icon: '🏆',
    color: '#f97316',
    description: 'Post-game screen. Overlay shows match result and performance breakdown.',
    overlayShown: true,
  },
};

// ---------------------------------------------------------------------------
// Demo interactive regions — registered for each phase to show the feature
// ---------------------------------------------------------------------------

const DEMO_REGIONS: Record<GamePhase, Rect[]> = {
  NOT_RUNNING: [],
  LAUNCHING:   [{ x: 300, y: 20, width: 500, height: 80 }],
  IN_LOBBY:    [{ x: 1500, y: 200, width: 400, height: 500 }],
  CHAMP_SELECT:[{ x: 10, y: 150, width: 260, height: 400 }, { x: 1650, y: 150, width: 260, height: 250 }],
  LOADING:     [{ x: 400, y: 960, width: 1100, height: 80 }],
  IN_GAME:     [{ x: 1700, y: 10, width: 210, height: 160 }, { x: 10, y: 900, width: 220, height: 130 }],
  END_GAME:    [{ x: 400, y: 200, width: 1100, height: 600 }],
};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

let logCounter = 0;

export default function DevDashboard() {
  const [currentPhase, setCurrentPhase]         = useState<GamePhase>('NOT_RUNNING');
  const [overlayVisible, setOverlayVisible]     = useState(false);
  const [opacity, setOpacity]                   = useState(1.0);
  const [captureVisible, setCaptureVisible]     = useState(false);
  const [activeRegions, setActiveRegions]       = useState<Rect[]>([]);
  const [config, setConfig]                     = useState<PlatformConfig | null>(null);
  const [log, setLog]                           = useState<LogEntry[]>([]);
  const [ipcReady, setIpcReady]                 = useState<boolean | null>(null); // null=checking, true=ok, false=error
  const logRef                                  = useRef<HTMLDivElement>(null);

  // Scroll event log to bottom on new entry
  useEffect(() => {
    logRef.current?.scrollTo({ top: logRef.current.scrollHeight, behavior: 'smooth' });
  }, [log]);

  const addLog = useCallback((type: LogEntry['type'], message: string) => {
    const now = new Date();
    const time = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`;
    setLog(prev => [...prev.slice(-99), { id: logCounter++, time, type, message }]);
  }, []);

  // ── On mount: load config, subscribe to events ──────────────────────────

  useEffect(() => {
    // Probe IPC immediately — this tells us if Tauri capabilities are working
    invoke<PlatformConfig>('get_platform_config')
      .then(cfg => {
        setConfig(cfg);
        setIpcReady(true);
        setOpacity(cfg.overlay.default_opacity);
        setCaptureVisible(cfg.overlay.screen_capture_visible);
        addLog('system', `IPC ready · config loaded (v${cfg.version})`);
      })
      .catch(err => {
        setIpcReady(false);
        addLog('error', `IPC NOT available: ${String(err)} — check capabilities config`);
      });

    // Load current phase (in case the app was re-opened mid-game)
    invoke<string>('get_current_phase')
      .then(phase => {
        setCurrentPhase(phase as GamePhase);
        addLog('system', `Current phase: ${phase}`);
      })
      .catch(() => {/* ignore */});

    addLog('system', 'Dev Simulator ready — select a game phase to begin');

    // Listen for phase-changed events (also emitted back to this window)
    const unlistenPromise = listen<string>('phase-changed', (event) => {
      addLog('event', `phase-changed → ${event.payload}`);
    });

    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, [addLog]);

  // ── Phase simulation ─────────────────────────────────────────────────────

  const simulatePhase = async (phase: GamePhase) => {
    // Optimistic update — UI responds immediately regardless of IPC result
    setCurrentPhase(phase);
    setOverlayVisible(PHASES[phase].overlayShown);
    setActiveRegions(DEMO_REGIONS[phase]);
    addLog('command', `simulate_phase("${phase}")`);

    try {
      await invoke('simulate_phase', { phase });

      // Register demo interactive regions for this phase
      const regions = DEMO_REGIONS[phase];
      await invoke('update_interactive_regions', { regions });
      addLog('system', `overlay ${PHASES[phase].overlayShown ? 'shown' : 'hidden'} · ${regions.length} interactive region(s)`);
    } catch (err) {
      addLog('error', `IPC failed: ${String(err)}`);
    }
  };

  // ── Overlay controls ─────────────────────────────────────────────────────

  const handleShowOverlay = async () => {
    try {
      addLog('command', 'invoke show_overlay()');
      await invoke('show_overlay');
      setOverlayVisible(true);
    } catch (err) {
      addLog('error', `show_overlay failed: ${err}`);
    }
  };

  const handleHideOverlay = async () => {
    try {
      addLog('command', 'invoke hide_overlay()');
      await invoke('hide_overlay');
      setOverlayVisible(false);
    } catch (err) {
      addLog('error', `hide_overlay failed: ${err}`);
    }
  };

  const handleOpacityChange = async (value: number) => {
    setOpacity(value);
    try {
      await invoke('set_overlay_opacity', { opacity: value });
      addLog('command', `set_overlay_opacity(${value.toFixed(2)})`);
    } catch (err) {
      addLog('error', `set_overlay_opacity failed: ${err}`);
    }
  };

  const handleCaptureToggle = async (visible: boolean) => {
    setCaptureVisible(visible);
    try {
      addLog('command', `set_screen_capture_visible(${visible})`);
      await invoke('set_screen_capture_visible', { visible });
    } catch (err) {
      addLog('error', `set_screen_capture_visible failed: ${err}`);
    }
  };

  const handleClearRegions = async () => {
    try {
      addLog('command', 'update_interactive_regions([]) — clearing all regions');
      await invoke('update_interactive_regions', { regions: [] });
      setActiveRegions([]);
    } catch (err) {
      addLog('error', `clear regions failed: ${err}`);
    }
  };

  const handleFetchRegions = async () => {
    try {
      const regions = await invoke<Rect[]>('get_interactive_regions');
      setActiveRegions(regions);
      addLog('command', `get_interactive_regions() → ${regions.length} region(s)`);
    } catch (err) {
      addLog('error', `get_interactive_regions failed: ${err}`);
    }
  };

  // ── Render ────────────────────────────────────────────────────────────────

  const phaseInfo = PHASES[currentPhase];

  return (
    <div className="dashboard">
      {/* ── Header ── */}
      <header className="dash-header">
        <div className="header-left">
          <span className="header-logo">🎮</span>
          <div>
            <h1 className="header-title">broken-latch Platform</h1>
            <p className="header-subtitle">Dev Simulator — v{config?.version ?? '...'}</p>
          </div>
        </div>
        <div className="header-status">
          <StatusBadge
            color={ipcReady === null ? '#f59e0b' : ipcReady ? '#22c55e' : '#ef4444'}
            label={ipcReady === null ? 'IPC: connecting…' : ipcReady ? 'IPC: ready' : 'IPC: ERROR — see log'}
          />
          <StatusBadge
            color={overlayVisible ? '#22c55e' : '#6b7280'}
            label={`Overlay: ${overlayVisible ? 'SHOWN' : 'HIDDEN'}`}
          />
          <StatusBadge
            color={phaseInfo.color}
            label={`Phase: ${phaseInfo.label}`}
          />
          <StatusBadge
            color={captureVisible ? '#f59e0b' : '#6b7280'}
            label={`OBS: ${captureVisible ? 'VISIBLE' : 'HIDDEN'}`}
          />
        </div>
      </header>

      <div className="dash-body">
        {/* ── Left column ── */}
        <div className="dash-left">

          {/* Phase simulation panel */}
          <section className="panel">
            <h2 className="panel-title">
              <span>🎯</span> Game Phase Simulation
            </h2>
            <p className="panel-desc">
              Click a phase to call <code>simulate_phase()</code> on the Rust backend.
              The overlay window reacts in real-time via the <code>phase-changed</code> event.
            </p>
            <div className="phase-grid">
              {(Object.keys(PHASES) as GamePhase[]).map(phase => (
                <button
                  key={phase}
                  className={`phase-btn ${currentPhase === phase ? 'active' : ''}`}
                  style={{
                    '--phase-color': PHASES[phase].color,
                  } as React.CSSProperties}
                  onClick={() => simulatePhase(phase)}
                >
                  <span className="phase-icon">{PHASES[phase].icon}</span>
                  <span className="phase-label">{PHASES[phase].label}</span>
                  {PHASES[phase].overlayShown
                    ? <span className="phase-overlay-tag">overlay on</span>
                    : <span className="phase-overlay-tag off">hidden</span>
                  }
                </button>
              ))}
            </div>
            {/* Current phase description */}
            <div className="phase-desc-box" style={{ borderColor: phaseInfo.color }}>
              <span style={{ color: phaseInfo.color }}>{phaseInfo.icon} {phaseInfo.label}</span>
              <p>{phaseInfo.description}</p>
            </div>
          </section>

          {/* Overlay controls */}
          <section className="panel">
            <h2 className="panel-title">
              <span>🪟</span> Overlay Controls
            </h2>
            <p className="panel-desc">
              Direct API calls that control the transparent fullscreen window.
            </p>

            <div className="control-row">
              <label className="control-label">Visibility</label>
              <div className="btn-pair">
                <button className="ctrl-btn green" onClick={handleShowOverlay}>
                  show_overlay()
                </button>
                <button className="ctrl-btn red" onClick={handleHideOverlay}>
                  hide_overlay()
                </button>
              </div>
            </div>

            <div className="control-row">
              <label className="control-label">
                Opacity — <code>set_overlay_opacity({opacity.toFixed(2)})</code>
              </label>
              <div className="slider-row">
                <span className="slider-val">0.0</span>
                <input
                  type="range"
                  min={0} max={1} step={0.05}
                  value={opacity}
                  onChange={e => handleOpacityChange(Number(e.target.value))}
                  className="slider"
                />
                <span className="slider-val">{opacity.toFixed(2)}</span>
              </div>
              <p className="control-hint">
                Uses Win32 <code>SetLayeredWindowAttributes(LWA_ALPHA)</code> on the overlay HWND.
              </p>
            </div>

            <div className="control-row">
              <label className="control-label">
                OBS / Screen Capture — <code>set_screen_capture_visible({String(captureVisible)})</code>
              </label>
              <label className="toggle-label">
                <input
                  type="checkbox"
                  checked={captureVisible}
                  onChange={e => handleCaptureToggle(e.target.checked)}
                />
                <span className="toggle-text">
                  {captureVisible
                    ? '✅ Overlay visible in OBS/Discord streams'
                    : '🚫 Overlay hidden from screen capture (WDA_EXCLUDEFROMCAPTURE)'}
                </span>
              </label>
            </div>
          </section>

          {/* Interactive regions */}
          <section className="panel">
            <h2 className="panel-title">
              <span>🖱️</span> Interactive Regions
            </h2>
            <p className="panel-desc">
              By default the overlay is fully click-through.
              <code>update_interactive_regions()</code> calls Win32 <code>SetWindowRgn()</code>
              to make specific Rect areas intercept mouse input — everything else passes
              clicks straight to the game.
            </p>
            <div className="btn-row">
              <button className="ctrl-btn" onClick={handleFetchRegions}>
                get_interactive_regions()
              </button>
              <button className="ctrl-btn red" onClick={handleClearRegions}>
                Clear all regions
              </button>
            </div>
            <div className="regions-list">
              {activeRegions.length === 0
                ? <p className="regions-empty">No active regions — overlay is fully click-through</p>
                : activeRegions.map((r, i) => (
                  <div key={i} className="region-item">
                    <code>Region {i + 1}</code>
                    <span>x:{r.x} y:{r.y} w:{r.width} h:{r.height}</span>
                  </div>
                ))
              }
            </div>
            <p className="control-hint">
              Regions are auto-set when you click a phase button above (demo values).
              In production, each app widget calls this with its own bounding box on mount/resize.
            </p>
          </section>
        </div>

        {/* ── Right column ── */}
        <div className="dash-right">

          {/* Platform config display */}
          <section className="panel">
            <h2 className="panel-title">
              <span>⚙️</span> Platform Config
            </h2>
            <p className="panel-desc">
              Loaded from <code>%APPDATA%/broken-latch/config.toml</code> at startup via
              <code>get_platform_config()</code>. This is the live config that drives platform behaviour.
            </p>
            {config
              ? (
                <div className="config-grid">
                  <ConfigRow group="startup" key_="launch_with_windows" value={String(config.startup.launch_with_windows)} />
                  <ConfigRow group="startup" key_="auto_update_check"   value={String(config.startup.auto_update_check)} />
                  <ConfigRow group="overlay" key_="default_opacity"     value={String(config.overlay.default_opacity)} />
                  <ConfigRow group="overlay" key_="screen_capture_visible" value={String(config.overlay.screen_capture_visible)} />
                  <ConfigRow group="performance" key_="performance_mode"   value={String(config.performance.performance_mode)} />
                  <ConfigRow group="performance" key_="cpu_usage_limit"    value={`${config.performance.cpu_usage_limit}%`} />
                  <ConfigRow group="developer" key_="debug_mode"           value={String(config.developer.debug_mode)} />
                  <ConfigRow group="developer" key_="show_console_logs"    value={String(config.developer.show_console_logs)} />
                  <ConfigRow group="developer" key_="simulated_game_phases" value={String(config.developer.simulated_game_phases)} />
                </div>
              )
              : <p className="config-loading">Loading config…</p>
            }
          </section>

          {/* Platform capabilities reference */}
          <section className="panel">
            <h2 className="panel-title">
              <span>📋</span> Platform API Reference
            </h2>
            <p className="panel-desc">
              All Tauri commands currently registered in <code>invoke_handler!</code>.
              Call these from any app via <code>invoke(commandName, args)</code>.
            </p>
            <div className="api-list">
              <ApiEntry cmd="show_overlay"               args=""                             desc="Show the fullscreen transparent overlay" />
              <ApiEntry cmd="hide_overlay"               args=""                             desc="Hide the overlay completely" />
              <ApiEntry cmd="set_overlay_opacity"        args="opacity: f32"                 desc="0.0–1.0 alpha via SetLayeredWindowAttributes" />
              <ApiEntry cmd="set_screen_capture_visible" args="visible: bool"                desc="Toggle WDA_EXCLUDEFROMCAPTURE (OBS visibility)" />
              <ApiEntry cmd="update_interactive_regions" args="regions: Rect[]"              desc="SetWindowRgn — define clickable widget areas" />
              <ApiEntry cmd="get_interactive_regions"    args=""                             desc="Returns currently active region array" />
              <ApiEntry cmd="simulate_phase"             args="phase: string"                desc="Emit phase-changed event + auto-manage overlay" />
              <ApiEntry cmd="get_current_phase"          args=""                             desc="Returns current GamePhase string" />
              <ApiEntry cmd="get_platform_config"        args=""                             desc="Returns full config.toml as JSON object" />
              <ApiEntry cmd="inject_hook"                args=""                             desc="⚠️ Legacy DLL injection (Vanguard-unsafe)" />
              <ApiEntry cmd="get_hook_status"            args=""                             desc="Returns DLL hook status string" />
            </div>
          </section>

          {/* Database schema reference */}
          <section className="panel">
            <h2 className="panel-title">
              <span>🗄️</span> Database Schema
            </h2>
            <p className="panel-desc">
              SQLite via <code>tauri-plugin-sql</code>. Schema defined in
              <code>migrations/001_initial.sql</code> and applied automatically at startup.
              Apps access storage via the platform's storage API (Task 09 SDK).
            </p>
            <div className="schema-list">
              <SchemaRow table="apps"             desc="App registry — id, name, version, manifest, auth_token, enabled" />
              <SchemaRow table="app_storage"      desc="Per-app key-value store — app_id, key, value" />
              <SchemaRow table="hotkeys"          desc="Global hotkey registry — app_id, hotkey_id, keys, win_hotkey_id" />
              <SchemaRow table="widget_positions" desc="Persisted widget layout — app_id, widget_id, x/y/width/height/opacity" />
              <SchemaRow table="webhooks"         desc="Event webhook subscriptions — app_id, event, url" />
            </div>
          </section>

          {/* Live event log */}
          <section className="panel panel-log">
            <h2 className="panel-title">
              <span>📡</span> Live Event Log
              <button className="clear-log-btn" onClick={() => setLog([])}>Clear</button>
            </h2>
            <div className="log-scroll" ref={logRef}>
              {log.length === 0
                ? <p className="log-empty">No events yet — click a phase button to begin</p>
                : log.map(entry => (
                  <div key={entry.id} className={`log-entry log-${entry.type}`}>
                    <span className="log-time">{entry.time}</span>
                    <span className="log-type">{entry.type}</span>
                    <span className="log-msg">{entry.message}</span>
                  </div>
                ))
              }
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Small helper components
// ---------------------------------------------------------------------------

function StatusBadge({ color, label }: { color: string; label: string }) {
  return (
    <span className="status-badge" style={{ borderColor: color, color }}>
      <span className="status-dot" style={{ background: color }} />
      {label}
    </span>
  );
}

function ConfigRow({ group, key_, value }: { group: string; key_: string; value: string }) {
  return (
    <div className="config-row">
      <span className="config-group">[{group}]</span>
      <span className="config-key">{key_}</span>
      <span className="config-val">{value}</span>
    </div>
  );
}

function ApiEntry({ cmd, args, desc }: { cmd: string; args: string; desc: string }) {
  return (
    <div className="api-entry">
      <code className="api-cmd">{cmd}</code>
      {args && <span className="api-args">({args})</span>}
      <span className="api-desc">{desc}</span>
    </div>
  );
}

function SchemaRow({ table, desc }: { table: string; desc: string }) {
  return (
    <div className="schema-row">
      <code className="schema-table">{table}</code>
      <span className="schema-desc">{desc}</span>
    </div>
  );
}
