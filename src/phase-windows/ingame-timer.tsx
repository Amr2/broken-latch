import React from 'react';
import ReactDOM from 'react-dom/client';
import OverlayFrame from '../platform/OverlayFrame';
import GameTimerWidget from '../components/widgets/GameTimerWidget';

// Overlay window — 140×50 px, positioned top-center, click_through: true.
// The drag edge is present but non-functional at Win32 level (WS_EX_TRANSPARENT
// means all input passes through regardless of HTML pointer-events).
ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <OverlayFrame>
      <GameTimerWidget />
    </OverlayFrame>
  </React.StrictMode>
);
