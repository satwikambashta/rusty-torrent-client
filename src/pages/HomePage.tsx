import { useState, useEffect } from "react";
import { Plus } from "lucide-react";
import { TorrentItem } from "../components/TorrentItem";
import { AddTorrentDialog } from "../components/AddTorrentDialog";
import { useAppStore } from "../services/store";
import {
  fetchTorrents,
  startTorrent,
  pauseTorrent,
  removeTorrent,
  addTorrent,
} from "../services/api";
import "./HomePage.css";

export function HomePage() {
  const torrents = useAppStore((state) => state.torrents);
  const setTorrents = useAppStore((state) => state.setTorrents);
  const setIsLoading = useAppStore((state) => state.setIsLoading);
  const isLoading = useAppStore((state) => state.isLoading);
  const [error, setError] = useState<string | null>(null);
  const [isDialogOpen, setIsDialogOpen] = useState(false);

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

  const handleAddTorrent = async (fileName: string) => {
    try {
      await addTorrent(fileName);
      setIsDialogOpen(false);
      setError(null);
      loadTorrents();
    } catch (err) {
      console.error("Failed to add torrent:", err);
      setError(`Failed to add torrent: ${err}`);
      throw err;
    }
  };

  return (
    <div className="home-page">
      <div className="page-header">
        <h2>Downloads</h2>
        <div className="header-actions">
          <button
            className="btn-secondary"
            onClick={() => setIsDialogOpen(true)}
            title="Add a new torrent"
          >
            <Plus size={18} />
            Add Torrent
          </button>
          <button className="btn-primary" onClick={loadTorrents} disabled={isLoading}>
            {isLoading ? "Loading..." : "Refresh"}
          </button>
        </div>
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

      <AddTorrentDialog
        isOpen={isDialogOpen}
        onClose={() => setIsDialogOpen(false)}
        onAdd={handleAddTorrent}
      />
    </div>
  );
}
