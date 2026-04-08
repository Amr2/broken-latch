/**
 * App.tsx — Main window root component
 * =====================================
 * The main Tauri window renders the Dev Simulator dashboard.
 * In production this would be the platform control panel / system tray UI.
 */
import DevDashboard from './components/DevDashboard/DevDashboard';

export default function App() {
  return <DevDashboard />;
}
