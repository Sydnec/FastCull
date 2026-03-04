import { useRef, useEffect, useState } from "react";
import type { FileInfo, PickStatus } from "../hooks/useCull";
import "../styles/gridview.css";

interface GridViewProps {
  files: FileInfo[];
  filteredIndices: number[];
  currentIndex: number;
  statuses: Map<number, PickStatus>;
  ratings: Map<number, number>;
  sessionId: number;
  onNavigate: (index: number) => void;
  onOpenSingle: (index: number) => void;
}

export function GridView({
  files,
  filteredIndices,
  currentIndex,
  statuses,
  ratings,
  sessionId,
  onNavigate,
  onOpenSingle,
}: GridViewProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  // Auto-scroll vers la miniature active
  useEffect(() => {
    const container = scrollRef.current;
    if (!container) return;
    const active = container.querySelector(".grid-thumb.active");
    if (active) {
      active.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }
  }, [currentIndex]);

  return (
    <div className="grid-view" ref={scrollRef}>
      {filteredIndices.map((fileIndex) => (
        <GridThumb
          key={fileIndex}
          index={fileIndex}
          filename={files[fileIndex]?.filename ?? ""}
          isCurrent={fileIndex === currentIndex}
          status={statuses.get(fileIndex) ?? "unrated"}
          rating={ratings.get(fileIndex) ?? 0}
          sessionId={sessionId}
          onClick={() => onNavigate(fileIndex)}
          onDoubleClick={() => onOpenSingle(fileIndex)}
        />
      ))}
    </div>
  );
}

function GridThumb({
  index,
  filename,
  isCurrent,
  status,
  rating,
  sessionId,
  onClick,
  onDoubleClick,
}: {
  index: number;
  filename: string;
  isCurrent: boolean;
  status: PickStatus;
  rating: number;
  sessionId: number;
  onClick: () => void;
  onDoubleClick: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setIsVisible(true);
          observer.disconnect();
        }
      },
      { rootMargin: "200px" },
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const borderClass =
    status === "pick"
      ? "status-border-pick"
      : status === "reject"
        ? "status-border-reject"
        : "";

  return (
    <div
      ref={ref}
      className={`grid-thumb ${isCurrent ? "active" : ""} ${borderClass}`}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
      title={filename}
    >
      {isVisible ? (
        <img
          src={`http://preview.localhost/${index}?s=${sessionId}`}
          alt={filename}
          loading="lazy"
          draggable={false}
        />
      ) : (
        <div className="grid-thumb-placeholder" />
      )}
      <div className="grid-thumb-overlay">
        <span className="grid-thumb-name">{filename}</span>
        <div className="grid-thumb-badges">
          {status !== "unrated" && (
            <span className={`thumb-badge status-${status}`}>
              {status === "pick" ? "R" : "X"}
            </span>
          )}
          {rating > 0 && (
            <span className="thumb-rating">{"\u2605".repeat(rating)}</span>
          )}
        </div>
      </div>
    </div>
  );
}
