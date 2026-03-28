import React from "react";
import { Server, Wifi, WifiOff } from "lucide-react";
import { useAppStore } from "../services/store";
import "./Header.css";

export const Header: React.FC = () => {
  const isConnected = useAppStore((state) => state.isConnected);

  return (
    <header className="web-header">
      <div className="header-content">
        <div className="header-title">
          <Server size={32} className="title-icon" />
          <h1>Rusty Torrents Monitor</h1>
        </div>

        <div className="header-status">
          <div className={`connection-badge ${isConnected ? "connected" : "disconnected"}`}>
            {isConnected ? (
              <>
                <Wifi size={16} />
                <span>Connected</span>
              </>
            ) : (
              <>
                <WifiOff size={16} />
                <span>Disconnected</span>
              </>
            )}
          </div>
        </div>
      </div>
    </header>
  );
};
