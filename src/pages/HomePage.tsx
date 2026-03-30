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
  listenDownloadProgress,
} from "../services/api";
import "./HomePage.css";

export function HomePage() {
  const torrents = useAppStore((state) => state.torrents);
  const setTorrents = useAppStore((state) => state.setTorrents);
  const setIsLoading = useAppStore((state) => state.setIsLoading);
  const isLoading = useAppStore((state) => state.isLoading);
  const [error, setError] = useState<string | null>(null);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [searchTerm, setSearchTerm] = useState<string>("");
  const [statusFilter, setStatusFilter] = useState<string>("All");
  const [sortKey, setSortKey] = useState<"name" | "size" | "progress" | "downloaded">("progress");
  const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");
  const [speedHistory, setSpeedHistory] = useState<number[]>([]);
  const [uploadSpeedHistory, setUploadSpeedHistory] = useState<number[]>([]);
  const [currentDownloadSpeed, setCurrentDownloadSpeed] = useState<number>(0);
  const [currentUploadSpeed, setCurrentUploadSpeed] = useState<number>(0);
  const [activePeers, setActivePeers] = useState<number>(0);
  const [totalPeers, setTotalPeers] = useState<number>(0);

  useEffect(() => {
    loadTorrents();

    let unlisten: (() => void) | null = null;
    listenDownloadProgress((payload) => {
      setCurrentDownloadSpeed(payload.stats.download_speed);
      setCurrentUploadSpeed(payload.stats.upload_speed);
      setActivePeers(payload.stats.peers_connected);
      setTotalPeers(payload.stats.total_peers);

      setSpeedHistory((curr) => {
        const next = [...curr.slice(-29), payload.stats.download_speed];
        return next;
      });

      setUploadSpeedHistory((curr) => {
        const next = [...curr.slice(-29), payload.stats.upload_speed];
        return next;
      });
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
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

  const filteredTorrents = torrents
    .filter((torrent) =>
      torrent.name.toLowerCase().includes(searchTerm.toLowerCase())
    )
    .filter((torrent) => statusFilter === "All" || torrent.status === statusFilter)
    .sort((a, b) => {
      const order = sortOrder === "asc" ? 1 : -1;
      if (sortKey === "name") {
        return a.name.localeCompare(b.name) * order;
      }
      if (sortKey === "size") {
        return (a.total_size - b.total_size) * order;
      }
      if (sortKey === "downloaded") {
        return (a.downloaded - b.downloaded) * order;
      }
      return (a.progress - b.progress) * order;
    });

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

      <section className="quick-metrics">
        <div className="metric-card">
          <h4>Download Speed</h4>
          <p>{currentDownloadSpeed} B/s</p>
        </div>
        <div className="metric-card">
          <h4>Upload Speed</h4>
          <p>{currentUploadSpeed} B/s</p>
        </div>
        <div className="metric-card">
          <h4>Active/Total Peers</h4>
          <p>{activePeers}/{totalPeers}</p>
        </div>
      </section>

      <section className="chart-panel">
        <div className="chart">
          <div className="chart-title">Download Speed (last 30s)</div>
          <div className="chart-bar-wrapper">
            {speedHistory.length === 0 ? (
              <span className="chart-empty">No data yet</span>
            ) : (
              speedHistory.map((value, idx) => (
                <div
                  key={`download-${idx}`}
                  className="chart-bar"
                  style={{
                    height: `${Math.min(200, value / 1024)}px`,
                    background: "#0ea5e9",
                  }}
                  title={`${value} B/s`}
                />
              ))
            )}
          </div>
        </div>
        <div className="chart">
          <div className="chart-title">Upload Speed (last 30s)</div>
          <div className="chart-bar-wrapper">
            {uploadSpeedHistory.length === 0 ? (
              <span className="chart-empty">No data yet</span>
            ) : (
              uploadSpeedHistory.map((value, idx) => (
                <div
                  key={`upload-${idx}`}
                  className="chart-bar"
                  style={{
                    height: `${Math.min(200, value / 1024)}px`,
                    background: "#22c55e",
                  }}
                  title={`${value} B/s`}
                />
              ))
            )}
          </div>
        </div>
      </section>

      <section className="filter-panel">
        <input
          type="text"
          placeholder="Filter by name"
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
        <select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value)}>
          <option value="All">All Statuses</option>
          <option value="Downloading">Downloading</option>
          <option value="Seeding">Seeding</option>
          <option value="Paused">Paused</option>
          <option value="Error">Error</option>
          <option value="Idle">Idle</option>
        </select>
        <select value={sortKey} onChange={(e) => setSortKey(e.target.value as any)}>
          <option value="progress">Progress</option>
          <option value="name">Name</option>
          <option value="size">Size</option>
          <option value="downloaded">Downloaded</option>
        </select>
        <select value={sortOrder} onChange={(e) => setSortOrder(e.target.value as "asc" | "desc")}>
          <option value="desc">Descending</option>
          <option value="asc">Ascending</option>
        </select>
      </section>

      {filteredTorrents.length === 0 ? (
        <div className="empty-state">
          <div className="empty-icon">📦</div>
          <p>No torrents yet. Add a torrent to get started!</p>
        </div>
      ) : (
        <div className="torrent-list">
          {filteredTorrents.map((torrent) => (
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
