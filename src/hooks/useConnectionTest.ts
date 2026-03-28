// Custom React hooks

import { useEffect, useState } from "react";
import { useAppStore } from "../services/store";

/**
 * Hook to initialize connection test on app load
 */
export function useConnectionTest() {
  const setIsConnected = useAppStore((state) => state.setIsConnected);
  const [isChecking, setIsChecking] = useState(true);

  useEffect(() => {
    const checkConnection = async () => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const response = await invoke("test_connection");
        if (response) {
          setIsConnected(true);
        }
      } catch (error) {
        console.error("Connection test failed:", error);
        setIsConnected(false);
      } finally {
        setIsChecking(false);
      }
    };

    checkConnection();
  }, [setIsConnected]);

  return { isChecking };
}
