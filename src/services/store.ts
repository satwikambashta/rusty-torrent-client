// Zustand global state management for the application
import { create } from "zustand";
import { TorrentInfo } from "../types";

interface AppState {
  // UI connection status
  isConnected: boolean;
  setIsConnected: (connected: boolean) => void;

  // Torrent list and operations
  torrents: TorrentInfo[];
  setTorrents: (torrents: TorrentInfo[]) => void;
  addTorrent: (torrent: TorrentInfo) => void;
  removeTorrent: (id: string) => void;
  updateTorrent: (id: string, updates: Partial<TorrentInfo>) => void;

  // Loading and error states
  isLoading: boolean;
  setIsLoading: (loading: boolean) => void;
  error: string | null;
  setError: (error: string | null) => void;
}

export const useAppStore = create<AppState>((set) => ({
  // Connection state
  isConnected: false,
  setIsConnected: (connected) => set({ isConnected: connected }),

  // Torrent management
  torrents: [],
  setTorrents: (torrents) => set({ torrents }),
  addTorrent: (torrent) =>
    set((state) => ({
      torrents: [...state.torrents, torrent],
    })),
  removeTorrent: (id) =>
    set((state) => ({
      torrents: state.torrents.filter((t) => t.id !== id),
    })),
  updateTorrent: (id, updates) =>
    set((state) => ({
      torrents: state.torrents.map((t) =>
        t.id === id ? { ...t, ...updates } : t
      ),
    })),

  // UI states
  isLoading: false,
  setIsLoading: (loading) => set({ isLoading: loading }),
  error: null,
  setError: (error) => set({ error }),
}));
