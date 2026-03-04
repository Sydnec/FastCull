import type { PickStatus } from "../hooks/useCull";
import "../styles/statusbar.css";

interface StatusBarProps {
  filename: string;
  currentIndex: number;
  total: number;
  picks: number;
  rejects: number;
  status: PickStatus;
  rating: number;
  onExportClick: () => void;
  onRatingChange: (rating: number) => void;
  onBack: () => void;
}

export function StatusBar({
  filename,
  currentIndex,
  total,
  picks,
  rejects,
  status,
  rating,
  onExportClick,
  onRatingChange,
  onBack,
}: StatusBarProps) {
  return (
    <div className="status-bar">
      <div className="status-bar-left">
        <button className="btn-back" onClick={onBack} title="Retour au navigateur">
          &larr;
        </button>
        <span className={`status-badge status-${status}`}>
          {status === "pick" ? "RET" : status === "reject" ? "REJ" : "---"}
        </span>
        <span className="status-filename" title={filename}>
          {filename}
        </span>
      </div>

      <div className="status-bar-center">
        <span>
          {total > 0 ? currentIndex + 1 : 0} / {total}
        </span>
      </div>

      <div className="status-bar-right">
        <span className="status-picks">Ret:{picks}</span>
        <span className="status-rejects">Rej:{rejects}</span>
        <span className="status-rating">
          {[1, 2, 3, 4, 5].map((r) => (
            <span
              key={r}
              className={`star ${r <= rating ? "filled" : "empty"}`}
              onClick={() => onRatingChange(r === rating ? 0 : r)}
            >
              {r <= rating ? "\u2605" : "\u2606"}
            </span>
          ))}
        </span>
        <button className="btn-export" onClick={onExportClick} title="Exporter (E)">
          Exporter
        </button>
      </div>
    </div>
  );
}
