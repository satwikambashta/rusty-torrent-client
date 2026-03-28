import { ConnectionStatus } from "./ConnectionStatus";
import "./Header.css";

export function Header() {
  return (
    <header className="app-header">
      <div className="header-content">
        <h1 className="app-title">📥 Rusty Torrents</h1>
        <ConnectionStatus />
      </div>
    </header>
  );
}
