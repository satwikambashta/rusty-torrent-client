import { useState } from "react";
import { Plus, X } from "lucide-react";
import "./AddTorrentDialog.css";

interface AddTorrentDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onAdd: (path: string) => Promise<void>;
}

export function AddTorrentDialog({ isOpen, onClose, onAdd }: AddTorrentDialogProps) {
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (files && files.length > 0) {
      const file = files[0];
      if (file.name.endsWith(".torrent")) {
        setSelectedFile(file.name);
        setError(null);
      } else {
        setError("Please select a valid .torrent file");
        setSelectedFile(null);
      }
    }
  };

  const handleAdd = async () => {
    if (!selectedFile) {
      setError("Please select a torrent file");
      return;
    }

    setIsLoading(true);
    try {
      await onAdd(selectedFile);
      setSelectedFile(null);
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to add torrent");
    } finally {
      setIsLoading(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="dialog-overlay">
      <div className="dialog">
        <div className="dialog-header">
          <h2>Add Torrent</h2>
          <button className="dialog-close" onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        <div className="dialog-content">
          <div className="file-input-wrapper">
            <input
              type="file"
              accept=".torrent"
              onChange={handleFileSelect}
              className="file-input"
              id="torrent-file"
            />
            <label htmlFor="torrent-file" className="file-label">
              <Plus size={20} />
              <span>Select .torrent file</span>
            </label>
          </div>

          {selectedFile && (
            <div className="selected-file">
              <p>Selected: {selectedFile}</p>
            </div>
          )}

          {error && <div className="error-message">{error}</div>}
        </div>

        <div className="dialog-footer">
          <button className="btn-secondary" onClick={onClose} disabled={isLoading}>
            Cancel
          </button>
          <button
            className="btn-primary"
            onClick={handleAdd}
            disabled={!selectedFile || isLoading}
          >
            {isLoading ? "Adding..." : "Add Torrent"}
          </button>
        </div>
      </div>
    </div>
  );
}
