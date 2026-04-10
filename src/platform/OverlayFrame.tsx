/**
 * OverlayFrame — Platform-injected wrapper for every overlay window.
 *
 * Responsibilities:
 *  1. Fetches this window's OverlayConfig from the backend via `get_overlay_config`.
 *  2. Injects a 4-pixel drag strip at the top of the window (when drag_edge: true).
 *     The strip carries `data-tauri-drag-region` so the platform handles dragging.
 *     Developers can also call `getCurrentWindow().startDragging()` from any element.
 *  3. Exposes the resolved config via the `useOverlayConfig` hook (optional).
 *
 * Usage — every overlay entry-point wraps its widget with OverlayFrame:
 * ```tsx
 * ReactDOM.createRoot(document.getElementById('root')!).render(
 *   <React.StrictMode>
 *     <OverlayFrame>
 *       <KdaWidget />
 *     </OverlayFrame>
 *   </React.StrictMode>
 * );
 * ```
 */

import { useState, useEffect, createContext, useContext } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import './overlay-frame.css';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface OverlayConfig {
  id:                     string;
  url:                    string;
  width:                  number | null;
  height:                 number | null;
  x:                      number | null;
  y:                      number | null;
  opacity:                number;
  click_through:          boolean;
  always_on_top:          boolean;
  screen_capture_visible: boolean;
  drag_edge:              boolean;
}

// ---------------------------------------------------------------------------
// Context — lets any child widget read its overlay config
// ---------------------------------------------------------------------------

const OverlayConfigContext = createContext<OverlayConfig | null>(null);

/** Read this overlay window's config from within any child component. */
export function useOverlayConfig(): OverlayConfig | null {
  return useContext(OverlayConfigContext);
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

interface OverlayFrameProps {
  children: React.ReactNode;
}

export default function OverlayFrame({ children }: OverlayFrameProps) {
  const [config, setConfig] = useState<OverlayConfig | null>(null);

  useEffect(() => {
    // Label format: "phase-{id}"  →  strip the prefix to get the config id.
    const label = getCurrentWindow().label;
    const id    = label.replace(/^phase-/, '');

    invoke<OverlayConfig | null>('get_overlay_config', { id })
      .then(cfg => setConfig(cfg))
      .catch(() => {
        // Running in browser dev mode — no Tauri backend.  Use safe defaults.
        setConfig(null);
      });
  }, []);

  // Default drag_edge to true — matches the Rust default.
  const showDragEdge = config === null ? true : config.drag_edge;

  return (
    <OverlayConfigContext.Provider value={config}>
      <div className="overlay-frame">
        {showDragEdge && (
          <div
            className="overlay-drag-edge"
            data-tauri-drag-region
            title="Drag to move this overlay"
          />
        )}
        <div className="overlay-content">
          {children}
        </div>
      </div>
    </OverlayConfigContext.Provider>
  );
}
