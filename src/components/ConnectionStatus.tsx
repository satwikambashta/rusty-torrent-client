import { Wifi, WifiOff } from "lucide-react";
import { useAppStore } from "../services/store";
import "./ConnectionStatus.css";

export function ConnectionStatus() {
  const isConnected = useAppStore((state) => state.isConnected);

  return (
    <div className={`connection-status ${isConnected ? "connected" : "disconnected"}`}>
      {isConnected ? (
        <>
          <Wifi size={16} className="status-icon" />
          <span>Connected</span>
        </>
      ) : (
        <>
          <WifiOff size={16} className="status-icon" />
          <span>Disconnected</span>
        </>
      )}
    </div>
  );
}
