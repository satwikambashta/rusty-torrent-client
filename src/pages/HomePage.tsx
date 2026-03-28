import { useState, useEffect } from "react";
import { TorrentItem } from "../components/TorrentItem";
import { useAppStore } from "../services/store";
import {
  fetchTorrents,
  startTorrent,
  pauseTorrent,
  removeTorrent,
} from "../services/api";
import "./HomePage.css";

export function HomePage() {
  const torrents = useAppStore((state) => state.torrents);
  const setTorrents = useAppStore((state) => state.setTorrents);
  const setIsLoading = useAppStore((state) => state.setIsLoading);
  const isLoading = useAppStore((state) => state.isLoading);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadTorrents();
  }, []);

  const loadTorrents = async () => {
    setIsLoading(true);
    try {
      const data = await fetchTorrents();
      setTorrents(data);
      setError(null);
    } catch (err) {
      console.error("Failed to load torrents:", err);
      setError("Failed to load torrents. Check backend connection.");
    } finally {
      setIsLoading(false);
    }
  };

  const handleStart = async (id: string) => {
    try {
      await startTorrent(id);
      loadTorrents();
    } catch (err) {
      console.error("Failed to start torrent:", err);
      setError("Failed to start torrent");
    }
  };

  const handlePause = async (id: string) => {
    try {
      await pauseTorrent(id);
      loadTorrents();
    } catch (err) {
      console.error("Failed to pause torrent:", err);
      setError("Failed to pause torrent");
    }
  };

  const handleRemove = async (id: string) => {
    try {
      await removeTorrent(id);
      loadTorrents();
    } catch (err) {
      console.error("Failed to remove torrent:", err);
      setError("Failed to remove torrent");
    }
  };

  return (
    <div className="home-page">
      <div className="page-header">
        <h2>Downloads</h2>
        <button className="btn-primary" onClick={loadTorrents} disabled={isLoading}>
          {isLoading ? "Loading..." : "Refresh"}
        </button>
      </div>

      {error && <div className="error-message">{error}</div>}

      {torrents.length === 0 ? (
        <div className="empty-state">
          <div className="empty-icon">📦</div>
          <p>No torrents yet. Add a torrent to get started!</p>
        </div>
      ) : (
        <div className="torrent-list">
          {torrents.map((torrent) => (
            <TorrentItem
              key={torrent.id}
              torrent={torrent}
              onStart={handleStart}
              onPause={handlePause}
              onRemove={handleRemove}
            />
          ))}
        </div>
      )}
    </div>
  );
}
