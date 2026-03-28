import { Download } from "lucide-react";
import { ConnectionStatus } from "./ConnectionStatus";
import "./Header.css";

export function Header() {
  return (
    <header className="app-header">
      <div className="header-content">
        <h1 className="app-title">
          <Download size={28} className="title-icon" />
          Rusty Torrents
        </h1>
        <ConnectionStatus />
      </div>
    </header>
  );
}
