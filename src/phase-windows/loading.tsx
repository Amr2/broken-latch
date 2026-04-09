import React from 'react';
import ReactDOM from 'react-dom/client';
import LoadingWindow from '../components/stages/Loading';
import '../index.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <LoadingWindow />
  </React.StrictMode>
);
