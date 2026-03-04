import { useState, useEffect, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "../styles/dropzone.css";

interface DropzoneProps {
  onPathsSelected: (paths: string[]) => Promise<void>;
  isLoading: boolean;
}

export function Dropzone({ onPathsSelected, isLoading }: DropzoneProps) {
  const [isDragging, setIsDragging] = useState(false);

  // Écouter les événements drag-drop natifs Tauri
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    getCurrentWindow()
      .onDragDropEvent((event) => {
        if (event.payload.type === "over") {
          setIsDragging(true);
        } else if (event.payload.type === "leave") {
          setIsDragging(false);
        } else if (event.payload.type === "drop") {
          setIsDragging(false);
          const paths = event.payload.paths;
          if (paths.length > 0) {
            onPathsSelected(paths);
          }
        }
      })
      .then((fn) => {
        unlisten = fn;
      });

    return () => {
      unlisten?.();
    };
  }, [onPathsSelected]);

  const handleOpenFolder = useCallback(async () => {
    if (isLoading) return;
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      onPathsSelected([selected]);
    }
  }, [isLoading, onPathsSelected]);

  const handleOpenFiles = useCallback(async () => {
    if (isLoading) return;
    const selected = await open({
      directory: false,
      multiple: true,
      filters: [
        {
          name: "Fichiers RAW",
          extensions: [
            "cr2", "cr3", "nef", "arw", "dng",
            "orf", "pef", "rw2", "raf", "srw", "3fr",
            "CR2", "CR3", "NEF", "ARW", "DNG",
            "ORF", "PEF", "RW2", "RAF", "SRW", "3FR",
          ],
        },
      ],
    });
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      if (paths.length > 0) {
        onPathsSelected(paths);
      }
    }
  }, [isLoading, onPathsSelected]);

  return (
    <div className="dropzone">
      <div className={`dropzone-content ${isDragging ? "dropzone-active" : ""}`}>
        {isLoading ? (
          <>
            <h1>Chargement...</h1>
            <p>Scan des fichiers en cours</p>
          </>
        ) : (
          <>
            <h1>FastCull</h1>
            <p>Glissez un dossier ou des fichiers RAW ici</p>
            <div className="dropzone-buttons">
              <button className="btn btn-primary" onClick={handleOpenFolder}>
                Ouvrir un dossier
              </button>
              <button className="btn btn-secondary" onClick={handleOpenFiles}>
                Choisir des fichiers
              </button>
            </div>
            <p className="dropzone-hint">
              Les sous-dossiers sont inclus automatiquement
              <br />
              Formats : CR2, CR3, NEF, ARW, DNG, ORF, PEF, RW2, RAF
            </p>
          </>
        )}
      </div>
    </div>
  );
}
