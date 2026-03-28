import React, { useState } from "react";
import { Search, Loader2, AlertCircle } from "lucide-react";
import { searchTorrents, SearchResult } from "../services/api";
import "./SearchPage.css";

interface SearchState {
  query: string;
  results: SearchResult[];
  loading: boolean;
  error: string | null;
}

export const SearchPage: React.FC = () => {
  const [state, setState] = useState<SearchState>({
    query: "",
    results: [],
    loading: false,
    error: null,
  });

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!state.query.trim()) return;

    setState((prev) => ({ ...prev, loading: true, error: null }));

    try {
      const results = await searchTorrents(state.query, 50);
      setState((prev) => ({
        ...prev,
        results,
        loading: false,
      }));
    } catch (err) {
      setState((prev) => ({
        ...prev,
        error: err instanceof Error ? err.message : "Search failed",
        loading: false,
      }));
    }
  };

  const handleAddTorrent = (_magnet: string, name: string) => {
    console.log("Adding torrent:", name);
    // TODO: Integrate with add torrent functionality
  };

  const formatSize = (bytes: number): string => {
    const units = ["B", "KB", "MB", "GB", "TB"];
    let size = bytes;
    let unitIndex = 0;
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    return `${size.toFixed(2)} ${units[unitIndex]}`;
  };

  return (
    <div className="search-page-container">
      <div className="search-header">
        <h1>Search Torrents</h1>
      </div>

      <form onSubmit={handleSearch} className="search-form">
        <div className="search-input-wrapper">
          <Search className="search-icon" size={20} />
          <input
            type="text"
            placeholder="Search for torrents..."
            value={state.query}
            onChange={(e) => setState((prev) => ({ ...prev, query: e.target.value }))}
            className="search-input"
          />
          <button
            type="submit"
            disabled={state.loading}
            className="search-button"
          >
            {state.loading ? <Loader2 size={18} className="spinner" /> : "Search"}
          </button>
        </div>
      </form>

      {state.error && (
        <div className="error-message">
          <AlertCircle size={20} />
          <span>{state.error}</span>
        </div>
      )}

      <div className="search-results">
        {state.results.length > 0 ? (
          <div className="results-list">
            <h2>{state.results.length} Results</h2>
            <table className="results-table">
              <thead>
                <tr>
                  <th style={{ width: "40%" }}>Name</th>
                  <th style={{ width: "15%" }}>Size</th>
                  <th style={{ width: "12%" }}>Seeders</th>
                  <th style={{ width: "12%" }}>Leechers</th>
                  <th style={{ width: "21%" }}>Action</th>
                </tr>
              </thead>
              <tbody>
                {state.results.map((result, idx) => (
                  <tr key={idx}>
                    <td>
                      <div className="torrent-name" title={result.name}>
                        {result.name}
                      </div>
                    </td>
                    <td>
                      <span className="torrent-size">{formatSize(result.size)}</span>
                    </td>
                    <td>
                      <span className="seed-count">{result.seeders}</span>
                    </td>
                    <td>
                      <span className="leech-count">{result.leechers}</span>
                    </td>
                    <td>
                      <button
                        className="action-button"
                        onClick={() => handleAddTorrent(result.magnet, result.name)}
                      >
                        + Add
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : state.query && !state.loading ? (
          <div className="no-results">
            <p>No results found for "{state.query}"</p>
          </div>
        ) : (
          <div className="search-placeholder">
            <Search size={48} />
            <p>Search for torrents to get started</p>
          </div>
        )}
      </div>
    </div>
  );
};
