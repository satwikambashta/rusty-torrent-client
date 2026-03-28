import { useState } from "react";
import { testConnection, getServerInfo } from "../services/api";
import { TestResponse } from "../types";
import "./TestPage.css";

export function TestPage() {
  const [testResult, setTestResult] = useState<TestResponse | null>(null);
  const [serverInfo, setServerInfo] = useState<TestResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleTestConnection = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await testConnection();
      setTestResult(result);
    } catch (err) {
      setError(`Connection test failed: ${err}`);
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handleGetServerInfo = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await getServerInfo();
      setServerInfo(result);
    } catch (err) {
      setError(`Failed to get server info: ${err}`);
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="test-page">
      <div className="test-container">
        <h2>Backend & Frontend Connection Test</h2>
        <p className="subtitle">Verify that the Tauri backend and React frontend are communicating properly</p>

        <div className="test-section">
          <h3>Connection Tests</h3>

          <div className="button-group">
            <button
              className="test-button"
              onClick={handleTestConnection}
              disabled={loading}
            >
              {loading ? "Testing..." : "Test Connection"}
            </button>
            <button
              className="test-button"
              onClick={handleGetServerInfo}
              disabled={loading}
            >
              {loading ? "Loading..." : "Get Server Info"}
            </button>
          </div>

          {error && <div className="error-box">{error}</div>}

          {testResult && (
            <div className="result-box success">
              <h4>✓ Connection Test Result</h4>
              <details>
                <summary>View Details</summary>
                <pre>{JSON.stringify(testResult, null, 2)}</pre>
              </details>
            </div>
          )}

          {serverInfo && (
            <div className="result-box success">
              <h4>✓ Server Info</h4>
              <details>
                <summary>View Details</summary>
                <pre>{JSON.stringify(serverInfo, null, 2)}</pre>
              </details>
            </div>
          )}
        </div>

        <div className="info-section">
          <h3>How This Works</h3>
          <ol>
            <li>Click "Test Connection" to send a message from React to Rust</li>
            <li>The Rust backend receives the Tauri command and responds</li>
            <li>React displays the response confirming bidirectional communication</li>
            <li>This proves your frontend and backend are properly integrated</li>
          </ol>
        </div>

        <div className="status-section">
          <h3>System Status</h3>
          <div className="status-grid">
            <div className="status-item">
              <span className="status-label">Frontend</span>
              <span className="status-value">✓ Running</span>
            </div>
            <div className="status-item">
              <span className="status-label">Backend</span>
              <span className="status-value">
                {testResult ? "✓ Running" : "○ Not tested"}
              </span>
            </div>
            <div className="status-item">
              <span className="status-label">Communication</span>
              <span className="status-value">
                {testResult ? "✓ Established" : "○ Not tested"}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
