import { useState, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import "../styles/filebrowser.css";

interface DriveInfo {
  letter: string;
  label: string;
  total_bytes: number;
}

interface DirEntryInfo {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  raw_count: number;
}

interface FileBrowserProps {
  onPathsSelected: (paths: string[]) => void;
  onClose: () => void;
  isLoading: boolean;
}

export function FileBrowser({
  onPathsSelected,
  onClose,
  isLoading,
}: FileBrowserProps) {
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [entries, setEntries] = useState<DirEntryInfo[]>([]);
  const [currentPath, setCurrentPath] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [recursive, setRecursive] = useState(true);

  useEffect(() => {
    invoke<DriveInfo[]>("list_drives")
      .then(setDrives)
      .catch((e) => setError(String(e)));
  }, []);

  const navigateTo = useCallback(async (path: string, rec?: boolean) => {
    setError(null);
    try {
      const result = await invoke<DirEntryInfo[]>("list_directory", {
        path,
        recursive: rec ?? recursive,
      });
      setEntries(result);
      setCurrentPath(path);
    } catch (e) {
      setError(String(e));
    }
  }, [recursive]);

  const goToDrives = useCallback(() => {
    setCurrentPath(null);
    setEntries([]);
    setError(null);
  }, []);

  const goUp = useCallback(() => {
    if (!currentPath) return;
    const parts = currentPath.split(/[\\/]/).filter(Boolean);
    if (parts.length <= 1) {
      goToDrives();
    } else {
      parts.pop();
      const parent = parts.join("\\");
      navigateTo(parts.length === 1 ? parent + "\\" : parent);
    }
  }, [currentPath, navigateTo, goToDrives]);

  const breadcrumbs = useMemo(() => {
    if (!currentPath) return null;
    const parts = currentPath.split(/[\\/]/).filter(Boolean);
    const crumbs: { label: string; path: string }[] = [];
    let accumulated = "";
    for (const part of parts) {
      accumulated = accumulated ? accumulated + "\\" + part : part;
      crumbs.push({
        label: part,
        path: accumulated.length === 2 ? accumulated + "\\" : accumulated,
      });
    }
    return crumbs;
  }, [currentPath]);

  const rawCount = useMemo(
    () => entries.reduce((sum, e) => sum + (e.is_dir ? e.raw_count : 1), 0),
    [entries],
  );

  // Re-fetch quand on toggle recursive
  useEffect(() => {
    if (currentPath) {
      navigateTo(currentPath, recursive);
    }
  }, [recursive]);

  const handleOpen = useCallback(() => {
    if (!currentPath || isLoading) return;
    onPathsSelected([currentPath]);
  }, [currentPath, isLoading, onPathsSelected]);

  const formatSize = (bytes: number): string => {
    if (bytes === 0) return "";
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} Ko`;
    if (bytes < 1024 * 1024 * 1024)
      return `${(bytes / (1024 * 1024)).toFixed(1)} Mo`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} Go`;
  };

  return (
    <div
      className="dialog-overlay"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="fb-dialog">
        {/* En-tête */}
        <div className="fb-dialog-header">
          <h2 className="dialog-title">Importer des photos</h2>
          <button className="fb-close-btn" onClick={onClose}>
            &times;
          </button>
        </div>

        {/* Navigation breadcrumb */}
        <div className="fb-nav">
          {breadcrumbs ? (
            <div className="fb-breadcrumb">
              <button className="fb-breadcrumb-btn" onClick={goToDrives}>
                Ordinateur
              </button>
              {breadcrumbs.map((crumb, i) => (
                <span key={i}>
                  <span className="fb-breadcrumb-sep">&rsaquo;</span>
                  <button
                    className="fb-breadcrumb-btn"
                    onClick={() => navigateTo(crumb.path)}
                  >
                    {crumb.label}
                  </button>
                </span>
              ))}
              <button
                className="fb-up-btn"
                onClick={goUp}
                title="Dossier parent"
              >
                ..
              </button>
            </div>
          ) : (
            <span className="fb-subtitle">Sélectionnez un lecteur</span>
          )}
        </div>

        {/* Erreur */}
        {error && <div className="fb-error">{error}</div>}

        {/* Contenu */}
        <div className="fb-content">
          {isLoading ? (
            <div className="fb-empty">Chargement...</div>
          ) : currentPath === null ? (
            <div className="fb-list">
              {drives.map((drive) => (
                <div
                  key={drive.letter}
                  className="fb-item fb-drive"
                  onClick={() => navigateTo(drive.letter + "\\")}
                >
                  <span className="fb-item-icon fb-icon-drive" />
                  <span className="fb-item-name">
                    {drive.label
                      ? `${drive.label} (${drive.letter})`
                      : drive.letter}
                  </span>
                </div>
              ))}
            </div>
          ) : (
            <div className="fb-list">
              {entries.length === 0 && !error && (
                <div className="fb-empty">Aucun fichier RAW ou dossier</div>
              )}
              {entries.map((entry) => (
                <div
                  key={entry.path}
                  className={`fb-item ${entry.is_dir ? "fb-dir" : "fb-file"}`}
                  onClick={() => {
                    if (entry.is_dir) navigateTo(entry.path);
                  }}
                >
                  <span
                    className={`fb-item-icon ${entry.is_dir ? "fb-icon-dir" : "fb-icon-file"}`}
                  />
                  <span className="fb-item-name">{entry.name}</span>
                  <span className="fb-item-meta">
                    {entry.is_dir
                      ? entry.raw_count > 0
                        ? `${entry.raw_count} RAW`
                        : ""
                      : formatSize(entry.size)}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Pied avec boutons */}
        <div className="fb-dialog-footer">
          <label className="fb-recursive-toggle">
            <input
              type="checkbox"
              checked={recursive}
              onChange={(e) => setRecursive(e.target.checked)}
            />
            Inclure les sous-dossiers
          </label>
          <div className="fb-footer-actions">
            <button className="btn btn-secondary" onClick={onClose}>
              Annuler
            </button>
            <button
              className="btn btn-primary"
              onClick={handleOpen}
              disabled={!currentPath || rawCount === 0 || isLoading}
            >
              {currentPath && rawCount > 0
                ? `Importer (${rawCount} RAW)`
                : "Importer"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
