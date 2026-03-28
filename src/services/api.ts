// Tauri IPC command wrappers for backend communication
import { invoke } from "@tauri-apps/api/core";
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
