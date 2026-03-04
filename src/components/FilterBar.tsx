import type { ViewFilter } from "../hooks/useCull";
import type { ViewMode } from "./Viewer";
import "../styles/filterbar.css";

interface FilterBarProps {
  filter: ViewFilter;
  onFilterChange: (filter: ViewFilter) => void;
  counts: {
    total: number;
    picks: number;
    rejects: number;
    unrated: number;
    filtered: number;
  };
  viewMode: ViewMode;
  onToggleView: () => void;
}

export function FilterBar({
  filter,
  onFilterChange,
  counts,
  viewMode,
  onToggleView,
}: FilterBarProps) {
  return (
    <div className="filter-bar">
      <div className="filter-bar-statuses">
        <button
          className={`filter-btn pick ${filter.showPick ? "active" : ""}`}
          onClick={() =>
            onFilterChange({ ...filter, showPick: !filter.showPick })
          }
        >
          Retenues ({counts.picks})
        </button>
        <button
          className={`filter-btn reject ${filter.showReject ? "active" : ""}`}
          onClick={() =>
            onFilterChange({ ...filter, showReject: !filter.showReject })
          }
        >
          Rejetées ({counts.rejects})
        </button>
        <button
          className={`filter-btn unrated ${filter.showUnrated ? "active" : ""}`}
          onClick={() =>
            onFilterChange({ ...filter, showUnrated: !filter.showUnrated })
          }
        >
          Non notées ({counts.unrated})
        </button>
      </div>
      <div className="filter-bar-separator" />
      <div className="filter-bar-rating">
        {[0, 1, 2, 3, 4, 5].map((r) => (
          <button
            key={r}
            className={`filter-rating-btn ${filter.minRating === r ? "active" : ""}`}
            onClick={() => onFilterChange({ ...filter, minRating: r })}
          >
            {r === 0 ? "Toutes" : "\u2605".repeat(r) + "+"}
          </button>
        ))}
      </div>
      <div className="filter-bar-separator" />
      <span className="filter-bar-count">
        {counts.filtered} / {counts.total}
      </span>
      <div className="filter-bar-separator" />
      <button
        className="filter-view-toggle"
        onClick={onToggleView}
        title="Basculer la vue (Tab)"
      >
        {viewMode === "single" ? "Grille" : "Photo"}
      </button>
    </div>
  );
}
