// HTTP API client for remote monitoring
import type {
  TorrentInfo,
  TorrentStats,
  HealthResponse,
  SeedingEvent,
  ApiError,
} from "../types";

const API_BASE = "/api";

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(
    endpoint: string,
    options?: RequestInit
  ): Promise<T> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        headers: {
          "Content-Type": "application/json",
          ...options?.headers,
        },
        ...options,
      });

      if (!response.ok) {
        const error: ApiError = {
          message: `HTTP ${response.status}: ${response.statusText}`,
          status: response.status,
        };
        throw error;
      }

      return await response.json() as T;
    } catch (error) {
      if (error instanceof Error) {
        throw {
          message: error.message,
          status: 0,
        } as ApiError;
      }
      throw error;
    }
  }

  async healthCheck(): Promise<HealthResponse> {
    return this.request<HealthResponse>("/health");
  }

  async getTorrents(): Promise<TorrentInfo[]> {
    return this.request<TorrentInfo[]>("/torrents");
  }

  async getTorrentStats(): Promise<TorrentStats> {
    return this.request<TorrentStats>("/torrents/stats");
  }

  async getPrioritizedTorrents(): Promise<TorrentInfo[]> {
    return this.request<TorrentInfo[]>("/torrents/prioritized");
  }

  async getSeedingEvents(): Promise<SeedingEvent[]> {
    return this.request<SeedingEvent[]>("/seeding-events");
  }

  async getRecentSeedingEvents(): Promise<SeedingEvent[]> {
    return this.request<SeedingEvent[]>("/seeding-events/recent");
  }
}

// Export singleton instance
export const apiClient = new ApiClient();
