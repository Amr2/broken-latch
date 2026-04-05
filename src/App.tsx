import React from 'react';

function App() {
  return (
    <div className="app-container">
      <div className="status-card">
        <h1 className="title">broken-latch Platform</h1>
        <p className="status-message">Initialization successful</p>
        <div className="info-section">
          <p>Platform Version: 0.1.0</p>
          <p>Framework: Tauri 2.0 + React</p>
        </div>
      </div>
    </div>
  );
}

export default App;
