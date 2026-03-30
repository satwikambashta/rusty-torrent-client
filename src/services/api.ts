// Tauri IPC command wrappers for backend communication
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { TorrentInfo, TestResponse, ServerInfo } from "../types";

// Connection & Server Commands

export async function testConnection(): Promise<TestResponse> {
  return invoke("test_connection");
}

export async function getServerInfo(): Promise<ServerInfo> {
  return invoke("get_server_info");
}

// Torrent Commands

export async function fetchTorrents(): Promise<TorrentInfo[]> {
  return invoke("get_torrents");
}

export async function addTorrent(path: string): Promise<string> {
  return invoke("add_torrent", { torrentPath: path });
}

export async function startTorrent(torrentId: string): Promise<void> {
  return invoke("start_torrent", { torrentId });
}

export async function pauseTorrent(torrentId: string): Promise<void> {
  return invoke("pause_torrent", { torrentId });
}

export async function removeTorrent(torrentId: string): Promise<void> {
  return invoke("remove_torrent", { torrentId });
}

// Search & Scan Commands

export interface SearchResult {
  name: string;
  size: number;
  seeders: number;
  leechers: number;
  magnet: string;
}

export interface ScannedFile {
  path: string;
  size: number;
  md5: string;
  sha1: string;
}

export async function searchTorrents(query: string, limit?: number): Promise<SearchResult[]> {
  return invoke("search_torrents", { query, limit: limit || 20 });
}

export async function scanFolder(path: string): Promise<ScannedFile[]> {
  return invoke("scan_folder", { path });
}

export interface AppConfig {
  download_dir: string;
  upload_rate_limit: number;
  download_rate_limit: number;
  max_connections: number;
  listen_port: number;
  web_ui_port: number;
  enable_file_logging: boolean;
  log_dir: string;
  seed_prioritization: number;
  max_seeding_torrents: number;
  auto_scan_folders: boolean;
  scan_folders: string[];
  min_seeders_threshold: number;
  verbose_logging: boolean;
}

export async function getConfig(): Promise<AppConfig> {
  return invoke("get_config");
}

export async function updateConfig(config: AppConfig): Promise<void> {
  return invoke("update_config", { configJson: config });
}

export interface SeedingStats {
  active_torrents: number;
  total_uploaded: number;
  avg_seeders: number;
  optimization_score: number;
}

export async function getSeedingStats(): Promise<SeedingStats> {
  return invoke("get_seeding_stats");
}

export interface DownloadProgressPayload {
  session_id: string;
  stats: {
    pieces: any;
    peers_connected: number;
    total_peers: number;
    download_speed: number;
    upload_speed: number;
    active_blocks: number;
  };
}

export function listenDownloadProgress(
  callback: (payload: DownloadProgressPayload) => void
): Promise<UnlistenFn> {
  return listen("download-progress", (event) => {
    callback(event.payload as DownloadProgressPayload);
  });
}

