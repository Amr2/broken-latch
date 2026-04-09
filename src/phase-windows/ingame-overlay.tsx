import React from 'react';
import ReactDOM from 'react-dom/client';
import InGame from '../components/stages/InGame';

// Overlay window — transparent fullscreen, pointer-events: none by default.
// Individual widget panels inside InGame use pointer-events: auto.
ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <InGame />
  </React.StrictMode>
);
