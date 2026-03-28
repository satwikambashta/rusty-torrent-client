import { useAppStore } from "../services/store";
import "./ConnectionStatus.css";

export function ConnectionStatus() {
  const isConnected = useAppStore((state) => state.isConnected);

  return (
    <div className={`connection-status ${isConnected ? "connected" : "disconnected"}`}>
      <div className="status-indicator"></div>
      <span>{isConnected ? "Connected" : "Disconnected"}</span>
    </div>
  );
}
