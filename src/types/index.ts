// Frontend type definitions

export interface TorrentInfo {
  id: string;
  name: string;
  total_size: number;
  downloaded: number;
  uploaded: number;
  status: "Idle" | "Downloading" | "Uploading" | "Seeding" | "Paused" | "Error";
  progress: number; // 0-100
  connected_peers?: number;
  total_peers?: number;
}

export interface TestResponse {
  status: string;
  message: string;
  timestamp: string;
  backend_version: string;
}

export interface ServerInfo {
  status: string;
  message: string;
  timestamp: string;
  backend_version: string;
}
