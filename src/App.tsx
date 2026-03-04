import { useState, useEffect, useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { FileBrowser } from "./components/FileBrowser";
import { Viewer } from "./components/Viewer";
import { useCull } from "./hooks/useCull";
import "./styles/app.css";

function App() {
  const cull = useCull();
  const [showImport, setShowImport] = useState(false);
  const [isDragging, setIsDragging] = useState(false);

  // Drag-drop natif Tauri — actif en permanence
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    getCurrentWindow()
      .onDragDropEvent((event) => {
        if (event.payload.type === "over") setIsDragging(true);
        else if (event.payload.type === "leave") setIsDragging(false);
        else if (event.payload.type === "drop") {
          setIsDragging(false);
          if (event.payload.paths.length > 0) {
            cull.openPaths(event.payload.paths);
          }
        }
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => {
      unlisten?.();
    };
  }, [cull.openPaths]);

  const handleImport = useCallback(
    (paths: string[]) => {
      setShowImport(false);
      cull.openPaths(paths);
    },
    [cull.openPaths],
  );

  return (
    <div className="app">
      {cull.files.length === 0 ? (
        <div className={`welcome ${isDragging ? "welcome-drag-active" : ""}`}>
          <div className="welcome-content">
            <h1 className="welcome-title">FastCull</h1>
            <p className="welcome-desc">Glissez un dossier de photos RAW ici</p>
            <button
              className="btn btn-primary welcome-import-btn"
              onClick={() => setShowImport(true)}
              disabled={cull.isLoading}
            >
              {cull.isLoading ? "Chargement..." : "Importer"}
            </button>
            <p className="welcome-hint">
              CR2, CR3, NEF, ARW, DNG, ORF, PEF, RW2, RAF
            </p>
          </div>
        </div>
      ) : (
        <Viewer cull={cull} />
      )}

      {showImport && (
        <FileBrowser
          onPathsSelected={handleImport}
          onClose={() => setShowImport(false)}
          isLoading={cull.isLoading}
        />
      )}
    </div>
  );
}

export default App;
