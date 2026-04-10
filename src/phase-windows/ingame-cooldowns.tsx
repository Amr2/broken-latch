import React from 'react';
import ReactDOM from 'react-dom/client';
import OverlayFrame from '../platform/OverlayFrame';
import CooldownWidget from '../components/widgets/CooldownWidget';

// Overlay window — 240×180 px, positioned bottom-left, click_through: false.
// OverlayFrame injects the 4px drag strip at the top.
ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <OverlayFrame>
      <CooldownWidget />
    </OverlayFrame>
  </React.StrictMode>
);
