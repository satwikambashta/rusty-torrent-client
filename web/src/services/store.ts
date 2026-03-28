// Global state management using Zustand
import { create } from "zustand";
import type { TorrentInfo, TorrentStats, SeedingEvent } from "../types";

interface AppStore {
  // Torrents
  torrents: TorrentInfo[];
  setTorrents: (torrents: TorrentInfo[]) => void;

  // Statistics
  stats: TorrentStats | null;
  setStats: (stats: TorrentStats) => void;

  // UI State
  isLoading: boolean;
  setIsLoading: (loading: boolean) => void;

  error: string | null;
  setError: (error: string | null) => void;

  // Events
  recentEvents: SeedingEvent[];
  setRecentEvents: (events: SeedingEvent[]) => void;

  // Connection
  isConnected: boolean;
  setIsConnected: (connected: boolean) => void;

  // Refresh interval
  refreshInterval: number;
  setRefreshInterval: (interval: number) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  // Torrents
  torrents: [],
  setTorrents: (torrents) => set({ torrents }),

  // Statistics
  stats: null,
  setStats: (stats) => set({ stats }),

  // UI State
  isLoading: false,
  setIsLoading: (isLoading) => set({ isLoading }),

  error: null,
  setError: (error) => set({ error }),

  // Events
  recentEvents: [],
  setRecentEvents: (recentEvents) => set({ recentEvents }),

  // Connection
  isConnected: false,
  setIsConnected: (isConnected) => set({ isConnected }),

  // Refresh interval (milliseconds)
  refreshInterval: 5000, // 5 seconds default
  setRefreshInterval: (refreshInterval) => set({ refreshInterval }),
}));
