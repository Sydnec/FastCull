import { useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import "../styles/exportdialog.css";
import type {
  ExportMode,
  ExportFilter,
  ExportResult,
  PickStatus,
} from "../hooks/useCull";

interface ExportDialogProps {
  total: number;
  statuses: Map<number, PickStatus>;
  ratings: Map<number, number>;
  onExport: (mode: ExportMode, filter: ExportFilter) => Promise<ExportResult>;
  onClose: () => void;
}

export function ExportDialog({
  total,
  statuses,
  ratings,
  onExport,
  onClose,
}: ExportDialogProps) {
  const [mode, setMode] = useState<ExportMode>("move");
  const [isExporting, setIsExporting] = useState(false);
  const [result, setResult] = useState<ExportResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Filtres
  const [filterPick, setFilterPick] = useState(true);
  const [filterReject, setFilterReject] = useState(false);
  const [filterUnrated, setFilterUnrated] = useState(false);
  const [minRating, setMinRating] = useState(0);

  // Comptages par statut
  const statusCounts = useMemo(() => {
    let picks = 0;
    let rejects = 0;
    for (const s of statuses.values()) {
      if (s === "pick") picks++;
      else if (s === "reject") rejects++;
    }
    return {
      picks,
      rejects,
      unrated: total - picks - rejects,
    };
  }, [statuses, total]);

  // Nombre de photos correspondant au filtre
  const filteredCount = useMemo(() => {
    const selectedStatuses: PickStatus[] = [];
    if (filterPick) selectedStatuses.push("pick");
    if (filterReject) selectedStatuses.push("reject");
    if (filterUnrated) selectedStatuses.push("unrated");

    let count = 0;
    for (let i = 0; i < total; i++) {
      const status = statuses.get(i) ?? "unrated";
      const rating = ratings.get(i) ?? 0;
      if (selectedStatuses.includes(status) && rating >= minRating) {
        count++;
      }
    }
    return count;
  }, [total, statuses, ratings, filterPick, filterReject, filterUnrated, minRating]);

  const buildFilter = (): ExportFilter => {
    const filterStatuses: PickStatus[] = [];
    if (filterPick) filterStatuses.push("pick");
    if (filterReject) filterStatuses.push("reject");
    if (filterUnrated) filterStatuses.push("unrated");
    return { statuses: filterStatuses, min_rating: minRating };
  };

  const handleExport = async () => {
    if (filteredCount === 0) return;
    setIsExporting(true);
    setError(null);
    try {
      const res = await onExport(mode, buildFilter());
      setResult(res);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsExporting(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      onClose();
    } else if (e.key === "Enter" && !isExporting && !result && filteredCount > 0) {
      handleExport();
    }
  };

  return (
    <div className="dialog-overlay" onKeyDown={handleKeyDown} tabIndex={-1}>
      <div className="dialog">
        {!result ? (
          <>
            <h2 className="dialog-title">Exporter les photos</h2>

            {/* Filtre par statut */}
            <div className="dialog-section">
              <h3 className="dialog-section-title">Filtrer par statut</h3>
              <div className="filter-checkboxes">
                <label className="filter-checkbox">
                  <input
                    type="checkbox"
                    checked={filterPick}
                    onChange={(e) => setFilterPick(e.target.checked)}
                  />
                  <span className="filter-label filter-pick">
                    Retenues ({statusCounts.picks})
                  </span>
                </label>
                <label className="filter-checkbox">
                  <input
                    type="checkbox"
                    checked={filterReject}
                    onChange={(e) => setFilterReject(e.target.checked)}
                  />
                  <span className="filter-label filter-reject">
                    Rejetées ({statusCounts.rejects})
                  </span>
                </label>
                <label className="filter-checkbox">
                  <input
                    type="checkbox"
                    checked={filterUnrated}
                    onChange={(e) => setFilterUnrated(e.target.checked)}
                  />
                  <span className="filter-label">
                    Non notées ({statusCounts.unrated})
                  </span>
                </label>
              </div>
            </div>

            {/* Filtre par note minimale */}
            <div className="dialog-section">
              <h3 className="dialog-section-title">Note minimale</h3>
              <div className="filter-rating">
                {[0, 1, 2, 3, 4, 5].map((r) => (
                  <button
                    key={r}
                    className={`rating-btn ${minRating === r ? "active" : ""}`}
                    onClick={() => setMinRating(r)}
                  >
                    {r === 0 ? "Toutes" : "★".repeat(r)}
                  </button>
                ))}
              </div>
            </div>

            {/* Résumé du filtre */}
            <div className="filter-summary">
              <span className="filter-count">
                {filteredCount} photo{filteredCount !== 1 ? "s" : ""} correspondante{filteredCount !== 1 ? "s" : ""}
              </span>
            </div>

            {filteredCount === 0 ? (
              <p className="dialog-warning">
                Aucune photo ne correspond aux filtres sélectionnés.
              </p>
            ) : (
              <>
                {/* Mode d'export */}
                <div className="dialog-section">
                  <h3 className="dialog-section-title">Mode d'export</h3>
                  <div className="dialog-modes">
                    <label
                      className={`mode-option ${mode === "move" ? "active" : ""}`}
                    >
                      <input
                        type="radio"
                        name="mode"
                        value="move"
                        checked={mode === "move"}
                        onChange={() => setMode("move")}
                      />
                      <div className="mode-info">
                        <span className="mode-name">Déplacer</span>
                        <span className="mode-desc">
                          Déplace les fichiers dans Selected/ + XMP
                        </span>
                      </div>
                    </label>

                    <label
                      className={`mode-option ${mode === "copy" ? "active" : ""}`}
                    >
                      <input
                        type="radio"
                        name="mode"
                        value="copy"
                        checked={mode === "copy"}
                        onChange={() => setMode("copy")}
                      />
                      <div className="mode-info">
                        <span className="mode-name">Copier</span>
                        <span className="mode-desc">
                          Copie les fichiers dans Selected/ + XMP
                        </span>
                      </div>
                    </label>

                    <label
                      className={`mode-option ${mode === "xmponly" ? "active" : ""}`}
                    >
                      <input
                        type="radio"
                        name="mode"
                        value="xmponly"
                        checked={mode === "xmponly"}
                        onChange={() => setMode("xmponly")}
                      />
                      <div className="mode-info">
                        <span className="mode-name">XMP uniquement</span>
                        <span className="mode-desc">
                          Génère les fichiers .xmp à côté des RAW (pas de
                          déplacement)
                        </span>
                      </div>
                    </label>
                  </div>
                </div>
              </>
            )}

            {error && <p className="dialog-error">{error}</p>}

            {/* Boutons */}
            <div className="dialog-actions">
              <button className="btn btn-secondary" onClick={onClose}>
                Annuler
              </button>
              <button
                className="btn btn-primary"
                onClick={handleExport}
                disabled={filteredCount === 0 || isExporting}
              >
                {isExporting
                  ? "Export en cours..."
                  : `Exporter ${filteredCount} photo${filteredCount !== 1 ? "s" : ""}`}
              </button>
            </div>
          </>
        ) : (
          <>
            <h2 className="dialog-title">Export terminé</h2>

            <div className="dialog-result">
              {result.exported_count > 0 && (
                <p>
                  {result.exported_count} fichier
                  {result.exported_count > 1 ? "s" : ""}{" "}
                  {mode === "move" ? "déplacé" : "copié"}
                  {result.exported_count > 1 ? "s" : ""}
                </p>
              )}
              <p>
                {result.xmp_count} fichier{result.xmp_count > 1 ? "s" : ""} XMP
                généré{result.xmp_count > 1 ? "s" : ""}
              </p>
              {result.output_dir && (
                <p className="dialog-output-dir">{result.output_dir}</p>
              )}
            </div>

            <div className="dialog-actions">
              {result.output_dir && (
                <button
                  className="btn btn-secondary"
                  onClick={() =>
                    invoke("open_in_explorer", { path: result.output_dir })
                  }
                >
                  Ouvrir dans l'Explorateur
                </button>
              )}
              <button className="btn btn-primary" onClick={onClose}>
                Fermer
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
