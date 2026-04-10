import React from 'react';
import ReactDOM from 'react-dom/client';
import OverlayFrame from '../platform/OverlayFrame';
import KdaWidget from '../components/widgets/KdaWidget';

// Overlay window — 240×200 px, positioned top-right, click_through: false.
// OverlayFrame injects the 4px drag strip at the top.
ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <OverlayFrame>
      <KdaWidget />
    </OverlayFrame>
  </React.StrictMode>
);
