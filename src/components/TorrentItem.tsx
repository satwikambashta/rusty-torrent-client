import { useState } from "react";
import { TorrentInfo } from "../types";
import "./TorrentItem.css";

interface TorrentItemProps {
  torrent: TorrentInfo;
  onStart: (id: string) => void;
  onPause: (id: string) => void;
  onRemove: (id: string) => void;
}

export function TorrentItem({
  torrent,
  onStart,
  onPause,
  onRemove,
}: TorrentItemProps) {
  const [isHovering, setIsHovering] = useState(false);

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case "Downloading":
        return "#007bff";
      case "Uploading":
        return "#28a745";
      case "Seeding":
        return "#17a2b8";
      case "Paused":
        return "#ffc107";
      case "Error":
        return "#dc3545";
      default:
        return "#6c757d";
    }
  };

  return (
    <div
      className="torrent-item"
      onMouseEnter={() => setIsHovering(true)}
      onMouseLeave={() => setIsHovering(false)}
    >
      <div className="torrent-header">
        <h3 className="torrent-name">{torrent.name}</h3>
        <span
          className="torrent-status"
          style={{ color: getStatusColor(torrent.status) }}
        >
          {torrent.status}
        </span>
      </div>

      <div className="torrent-info">
        <div className="info-item">
          <span className="label">Size:</span>
          <span>{formatBytes(torrent.total_size)}</span>
        </div>
        <div className="info-item">
          <span className="label">Downloaded:</span>
          <span>{formatBytes(torrent.downloaded)}</span>
        </div>
        <div className="info-item">
          <span className="label">Uploaded:</span>
          <span>{formatBytes(torrent.uploaded)}</span>
        </div>
        <div className="info-item">
          <span className="label">Peers:</span>
          <span>
            {torrent.connected_peers ?? 0}/{torrent.total_peers ?? 0}
          </span>
        </div>
      </div>

      <div className="progress-bar">
        <div
          className="progress-fill"
          style={{ width: `${torrent.progress}%` }}
        ></div>
      </div>
      <div className="progress-text">{torrent.progress.toFixed(1)}%</div>

      {isHovering && (
        <div className="torrent-actions">
          {torrent.status !== "Downloading" && (
            <button
              className="btn btn-start"
              onClick={() => onStart(torrent.id)}
            >
              Start
            </button>
          )}
          {torrent.status === "Downloading" && (
            <button
              className="btn btn-pause"
              onClick={() => onPause(torrent.id)}
            >
              Pause
            </button>
          )}
          <button
            className="btn btn-remove"
            onClick={() => onRemove(torrent.id)}
          >
            Remove
          </button>
        </div>
      )}
    </div>
  );
}
