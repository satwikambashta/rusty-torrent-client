import React, { useEffect, useState } from "react";
import { Settings, Save, Loader2 } from "lucide-react";
import { getConfig, updateConfig, AppConfig } from "../services/api";
import "./ConfigPage.css";

export const ConfigPage: React.FC = () => {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const cfg = await getConfig();
      setConfig(cfg);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load config");
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    if (!config) return;

    setSaving(true);
    try {
      await updateConfig(config);
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save config");
    } finally {
      setSaving(false);
    }
  };

  const handleConfigChange = (key: keyof AppConfig, value: any) => {
    setConfig((prev) => {
      if (!prev) return prev;
      return { ...prev, [key]: value };
    });
  };

  if (loading) {
    return (
      <div className="config-page-container loading">
        <Loader2 className="spinner" size={40} />
        <p>Loading configuration...</p>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="config-page-container error">
        <p>Failed to load configuration</p>
      </div>
    );
  }

  return (
    <div className="config-page-container">
      <div className="config-header">
        <Settings size={32} />
        <h1>Configuration</h1>
        <p>Customize your torrent client settings</p>
      </div>

      {error && <div className="alert alert-error">{error}</div>}
      {success && (
        <div className="alert alert-success">Configuration saved successfully!</div>
      )}

      <div className="config-form">
        {/* Network Settings */}
        <section className="config-section">
          <h2>🌐 Network Settings</h2>
          <div className="form-group">
            <label>Listen Port</label>
            <input
              type="number"
              value={config.listen_port}
              onChange={(e) =>
                handleConfigChange("listen_port", parseInt(e.target.value))
              }
              min="1024"
              max="65535"
            />
          </div>
          <div className="form-group">
            <label>Web UI Port</label>
            <input
              type="number"
              value={config.web_ui_port}
              onChange={(e) =>
                handleConfigChange("web_ui_port", parseInt(e.target.value))
              }
              min="1024"
              max="65535"
            />
          </div>
          <div className="form-group">
            <label>Max Connections</label>
            <input
              type="number"
              value={config.max_connections}
              onChange={(e) =>
                handleConfigChange("max_connections", parseInt(e.target.value))
              }
              min="1"
            />
          </div>
        </section>

        {/* Rate Limiting */}
        <section className="config-section">
          <h2>⚡ Rate Limiting</h2>
          <div className="form-group">
            <label>Upload Limit (KB/s) - 0 for unlimited</label>
            <input
              type="number"
              value={config.upload_rate_limit}
              onChange={(e) =>
                handleConfigChange("upload_rate_limit", parseInt(e.target.value))
              }
              min="0"
            />
          </div>
          <div className="form-group">
            <label>Download Limit (KB/s) - 0 for unlimited</label>
            <input
              type="number"
              value={config.download_rate_limit}
              onChange={(e) =>
                handleConfigChange(
                  "download_rate_limit",
                  parseInt(e.target.value)
                )
              }
              min="0"
            />
          </div>
        </section>

        {/* Seeding Optimization */}
        <section className="config-section">
          <h2>🌱 Seeding Optimization</h2>
          <div className="form-group">
            <label>Seed Prioritization (0-100) - 0 to disable</label>
            <input
              type="range"
              value={config.seed_prioritization}
              onChange={(e) =>
                handleConfigChange("seed_prioritization", parseInt(e.target.value))
              }
              min="0"
              max="100"
            />
            <span className="value-display">{config.seed_prioritization}%</span>
          </div>
          <div className="form-group">
            <label>Max Seeding Torrents</label>
            <input
              type="number"
              value={config.max_seeding_torrents}
              onChange={(e) =>
                handleConfigChange(
                  "max_seeding_torrents",
                  parseInt(e.target.value)
                )
              }
              min="1"
            />
          </div>
          <div className="form-group">
            <label>Min Seeders Threshold</label>
            <input
              type="number"
              value={config.min_seeders_threshold}
              onChange={(e) =>
                handleConfigChange(
                  "min_seeders_threshold",
                  parseInt(e.target.value)
                )
              }
              min="0"
            />
          </div>
        </section>

        {/* Logging */}
        <section className="config-section">
          <h2>📝 Logging</h2>
          <div className="form-group checkbox">
            <input
              type="checkbox"
              id="enable-file-logging"
              checked={config.enable_file_logging}
              onChange={(e) =>
                handleConfigChange("enable_file_logging", e.target.checked)
              }
            />
            <label htmlFor="enable-file-logging">Enable File Logging</label>
          </div>
          <div className="form-group checkbox">
            <input
              type="checkbox"
              id="verbose-logging"
              checked={config.verbose_logging}
              onChange={(e) =>
                handleConfigChange("verbose_logging", e.target.checked)
              }
            />
            <label htmlFor="verbose-logging">Verbose Logging (Debug Level)</label>
          </div>
          <div className="form-group">
            <label>Log Directory</label>
            <input
              type="text"
              value={config.log_dir}
              onChange={(e) => handleConfigChange("log_dir", e.target.value)}
            />
          </div>
        </section>

        {/* Directory Settings */}
        <section className="config-section">
          <h2>📂 Directory Settings</h2>
          <div className="form-group">
            <label>Download Directory</label>
            <input
              type="text"
              value={config.download_dir}
              onChange={(e) =>
                handleConfigChange("download_dir", e.target.value)
              }
            />
          </div>
        </section>

        {/* Auto Scan */}
        <section className="config-section">
          <h2>🔍 Auto Scan</h2>
          <div className="form-group checkbox">
            <input
              type="checkbox"
              id="auto-scan"
              checked={config.auto_scan_folders}
              onChange={(e) =>
                handleConfigChange("auto_scan_folders", e.target.checked)
              }
            />
            <label htmlFor="auto-scan">Auto-scan folders on startup</label>
          </div>
        </section>

        {/* Save Button */}
        <div className="form-actions">
          <button
            className="btn-save"
            onClick={handleSave}
            disabled={saving}
          >
            {saving ? (
              <>
                <Loader2 size={20} className="spinner" />
                Saving...
              </>
            ) : (
              <>
                <Save size={20} />
                Save Configuration
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};
