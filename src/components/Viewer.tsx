import { useState, useEffect, useCallback } from "react";
import type { CullState } from "../hooks/useCull";
import { useKeyboard } from "../hooks/useKeyboard";
import { StatusBar } from "./StatusBar";
import { FilterBar } from "./FilterBar";
import { Filmstrip } from "./Filmstrip";
import { GridView } from "./GridView";
import { ExportDialog } from "./ExportDialog";
import "../styles/viewer.css";

export type ViewMode = "single" | "grid";

interface ViewerProps {
  cull: CullState;
}

export function Viewer({ cull }: ViewerProps) {
  const {
    files,
    currentIndex,
    currentFile,
    currentStatus,
    currentRating,
    counts,
    statuses,
    ratings,
    filter,
    setFilter,
    filteredIndices,
    sessionId,
    navigate,
    next,
    prev,
    setPickStatus,
    setRating,
    exportSelected,
  } = cull;

  const [showExport, setShowExport] = useState(false);
  const [viewMode, setViewMode] = useState<ViewMode>("single");
  const [filmstripCollapsed, setFilmstripCollapsed] = useState(false);

  const openExport = useCallback(() => {
    setShowExport(true);
  }, []);

  const closeExport = useCallback(() => {
    setShowExport(false);
  }, []);

  const toggleView = useCallback(() => {
    setViewMode((m) => (m === "single" ? "grid" : "single"));
  }, []);

  const toggleFilmstrip = useCallback(() => {
    setFilmstripCollapsed((c) => !c);
  }, []);

  useKeyboard({
    next,
    prev,
    setPickStatus,
    setRating,
    openExport,
    closeExport,
    toggleView,
  });

  // Image preloading (vue simple) — avec annulation pour navigation rapide
  const [displaySrc, setDisplaySrc] = useState("");
  const [isImageLoading, setIsImageLoading] = useState(false);

  const targetSrc = currentFile
    ? `http://preview.localhost/${currentIndex}?s=${sessionId}`
    : "";

  useEffect(() => {
    if (!targetSrc) {
      setDisplaySrc("");
      return;
    }
    if (targetSrc === displaySrc) return;

    let cancelled = false;
    setIsImageLoading(true);

    const img = new Image();
    img.src = targetSrc;

    // decode() est non-bloquant et ne gèle pas le thread UI
    img.decode().then(
      () => {
        if (!cancelled) {
          setDisplaySrc(targetSrc);
          setIsImageLoading(false);
        }
      },
      () => {
        // Erreur de décodage — afficher quand même
        if (!cancelled) {
          setDisplaySrc(targetSrc);
          setIsImageLoading(false);
        }
      },
    );

    return () => {
      cancelled = true;
    };
  }, [targetSrc]);

  const filterBarCounts = {
    total: counts.total,
    picks: counts.picks,
    rejects: counts.rejects,
    unrated: counts.total - counts.picks - counts.rejects,
    filtered: filteredIndices.length,
  };

  return (
    <div className="viewer">
      <FilterBar
        filter={filter}
        onFilterChange={setFilter}
        counts={filterBarCounts}
        viewMode={viewMode}
        onToggleView={toggleView}
      />

      {viewMode === "single" ? (
        <div className={`viewer-image-container ${currentStatus}`}>
          {displaySrc ? (
            <img
              className={`viewer-photo ${isImageLoading ? "loading" : ""}`}
              src={displaySrc}
              alt={currentFile?.filename ?? ""}
              draggable={false}
            />
          ) : (
            <div className="viewer-empty">Aucune photo</div>
          )}

          {currentStatus !== "unrated" && (
            <div className={`viewer-status-overlay status-${currentStatus}`}>
              {currentStatus === "pick" ? "RETENUE" : "REJETÉE"}
            </div>
          )}

          <div className="viewer-rating-overlay">
            {"★".repeat(currentRating)}
            {"☆".repeat(5 - currentRating)}
          </div>
        </div>
      ) : (
        <GridView
          files={files}
          filteredIndices={filteredIndices}
          currentIndex={currentIndex}
          statuses={statuses}
          ratings={ratings}
          sessionId={sessionId}
          onNavigate={navigate}
          onOpenSingle={(index) => {
            navigate(index);
            setViewMode("single");
          }}
        />
      )}

      <button
        className={`filmstrip-toggle ${filmstripCollapsed ? "collapsed" : ""}`}
        onClick={toggleFilmstrip}
        title={filmstripCollapsed ? "Afficher la pellicule" : "Masquer la pellicule"}
      >
        <span className="filmstrip-toggle-chevron">
          {filmstripCollapsed ? "\u25B2" : "\u25BC"}
        </span>
      </button>

      <div className={`filmstrip-wrapper ${filmstripCollapsed ? "collapsed" : ""}`}>
        <Filmstrip
          files={files}
          filteredIndices={filteredIndices}
          currentIndex={currentIndex}
          statuses={statuses}
          ratings={ratings}
          sessionId={sessionId}
          onNavigate={navigate}
        />
      </div>

      <StatusBar
        filename={currentFile?.filename ?? ""}
        currentIndex={currentIndex}
        total={counts.total}
        picks={counts.picks}
        rejects={counts.rejects}
        status={currentStatus}
        rating={currentRating}
        onExportClick={() => setShowExport(true)}
        onRatingChange={(r) => setRating(r)}
        onBack={cull.resetSession}
      />

      {showExport && (
        <ExportDialog
          total={counts.total}
          statuses={statuses}
          ratings={ratings}
          onExport={exportSelected}
          onClose={() => setShowExport(false)}
        />
      )}
    </div>
  );
}
