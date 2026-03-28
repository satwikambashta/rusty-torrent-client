import React, { useEffect, useState } from "react";
import {
  Download,
  Upload,
  Loader2,
  AlertCircle,
  RefreshCw,
  Activity,
} from "lucide-react";
import { apiClient } from "../services/api";
import { useAppStore } from "../services/store";
import type { TorrentInfo, TorrentStats } from "../types";
import "./Dashboard.css";

export const Dashboard: React.FC = () => {
  const [torrents, setTorrents] = useState<TorrentInfo[]>([]);
  const [stats, setStats] = useState<TorrentStats | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);

  const refreshInterval = useAppStore((state) => state.refreshInterval);
  const setIsConnected = useAppStore((state) => state.setIsConnected);

  const loadData = async () => {
    try {
      setError(null);
      const [torrents, stats] = await Promise.all([
        apiClient.getTorrents(),
        apiClient.getTorrentStats(),
      ]);
      setTorrents(torrents);
      setStats(stats);
      setIsConnected(true);
      setLastUpdated(new Date());
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to fetch data";
      setError(message);
      setIsConnected(false);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, refreshInterval);
    return () => clearInterval(interval);
  }, [refreshInterval, setIsConnected]);

  const formatBytes = (bytes: number): string => {
    const units = ["B", "KB", "MB", "GB", "TB"];
    let size = bytes;
    let unitIndex = 0;
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    return `${size.toFixed(2)} ${units[unitIndex]}`;
  };

  const formatTime = (date: Date): string => {
    return date.toLocaleTimeString();
  };

  return (
    <div className="dashboard">
      <div className="dashboard-header">
        <h2>Live Monitoring Dashboard</h2>
        <div className="header-actions">
          <button
            className="btn-refresh"
            onClick={loadData}
            disabled={isLoading}
            title="Refresh data"
          >
            <RefreshCw size={18} className={isLoading ? "spinning" : ""} />
            Refresh
          </button>
          {lastUpdated && (
            <span className="last-updated">
              Updated: {formatTime(lastUpdated)}
            </span>
          )}
        </div>
      </div>

      {error && (
        <div className="error-alert">
          <AlertCircle size={20} />
          <span>{error}</span>
        </div>
      )}

      {isLoading && torrents.length === 0 ? (
        <div className="loading-state">
          <Loader2 size={48} className="spinning" />
          <p>Connecting to backend...</p>
        </div>
      ) : (
        <>
          {/* Statistics Cards */}
          {stats && (
            <div className="stats-grid">
              <div className="stat-card">
                <div className="stat-icon torrents">
                  <Activity size={24} />
                </div>
                <div className="stat-content">
                  <p className="stat-label">Total Torrents</p>
                  <p className="stat-value">{stats.total_torrents}</p>
                </div>
              </div>

              <div className="stat-card">
                <div className="stat-icon seeding">
                  <Upload size={24} />
                </div>
                <div className="stat-content">
                  <p className="stat-label">Seeding</p>
                  <p className="stat-value">{stats.seeding}</p>
                </div>
              </div>

              <div className="stat-card">
                <div className="stat-icon downloading">
                  <Download size={24} />
                </div>
                <div className="stat-content">
                  <p className="stat-label">Downloading</p>
                  <p className="stat-value">{stats.downloading}</p>
                </div>
              </div>

              <div className="stat-card">
                <div className="stat-icon uploaded">
                  <Upload size={24} />
                </div>
                <div className="stat-content">
                  <p className="stat-label">Total Uploaded</p>
                  <p className="stat-value">{formatBytes(stats.total_uploaded)}</p>
                </div>
              </div>

              <div className="stat-card">
                <div className="stat-icon downloaded">
                  <Download size={24} />
                </div>
                <div className="stat-content">
                  <p className="stat-label">Total Downloaded</p>
                  <p className="stat-value">{formatBytes(stats.total_downloaded)}</p>
                </div>
              </div>
            </div>
          )}

          {/* Torrents Table */}
          {torrents.length > 0 ? (
            <div className="torrents-section">
              <h3>Active Torrents ({torrents.length})</h3>
              <table className="torrents-table">
                <thead>
                  <tr>
                    <th style={{ width: "35%" }}>Name</th>
                    <th style={{ width: "15%" }}>Size</th>
                    <th style={{ width: "15%" }}>Status</th>
                    <th style={{ width: "10%" }}>Progress</th>
                    <th style={{ width: "12%" }}>Downloaded</th>
                    <th style={{ width: "13%" }}>Uploaded</th>
                  </tr>
                </thead>
                <tbody>
                  {torrents.map((torrent) => (
                    <tr key={torrent.id}>
                      <td>
                        <span className="torrent-name" title={torrent.name}>
                          {torrent.name}
                        </span>
                      </td>
                      <td className="size">{formatBytes(torrent.total_size)}</td>
                      <td>
                        <span className={`status-badge ${torrent.status.toLowerCase()}`}>
                          {torrent.status}
                        </span>
                      </td>
                      <td>
                        <div className="progress-container">
                          <div className="progress-bar">
                            <div
                              className="progress-fill"
                              style={{ width: `${torrent.progress}%` }}
                            />
                          </div>
                          <span className="progress-text">{torrent.progress.toFixed(0)}%</span>
                        </div>
                      </td>
                      <td className="size">{formatBytes(torrent.downloaded)}</td>
                      <td className="size">{formatBytes(torrent.uploaded)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <div className="empty-state">
              <p>No active torrents</p>
            </div>
          )}
        </>
      )}
    </div>
  );
};
