import { useRef, useEffect, useState } from "react";
import type { FileInfo, PickStatus } from "../hooks/useCull";
import "../styles/filmstrip.css";

interface FilmstripProps {
  files: FileInfo[];
  filteredIndices: number[];
  currentIndex: number;
  statuses: Map<number, PickStatus>;
  ratings: Map<number, number>;
  sessionId: number;
  onNavigate: (index: number) => void;
}

export function Filmstrip({
  files,
  filteredIndices,
  currentIndex,
  statuses,
  ratings,
  sessionId,
  onNavigate,
}: FilmstripProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  // Auto-scroll vers la miniature active
  useEffect(() => {
    const container = scrollRef.current;
    if (!container) return;
    const active = container.querySelector(".filmstrip-thumb.active");
    if (active) {
      active.scrollIntoView({
        behavior: "smooth",
        block: "nearest",
        inline: "center",
      });
    }
  }, [currentIndex]);

  return (
    <div className="filmstrip">
      <div className="filmstrip-scroll" ref={scrollRef}>
        {filteredIndices.map((fileIndex) => (
          <FilmstripThumb
            key={fileIndex}
            index={fileIndex}
            filename={files[fileIndex]?.filename ?? ""}
            isCurrent={fileIndex === currentIndex}
            status={statuses.get(fileIndex) ?? "unrated"}
            rating={ratings.get(fileIndex) ?? 0}
            sessionId={sessionId}
            onClick={() => onNavigate(fileIndex)}
          />
        ))}
      </div>
    </div>
  );
}

function FilmstripThumb({
  index,
  filename,
  isCurrent,
  status,
  rating,
  sessionId,
  onClick,
}: {
  index: number;
  filename: string;
  isCurrent: boolean;
  status: PickStatus;
  rating: number;
  sessionId: number;
  onClick: () => void;
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
      { rootMargin: "300px 300px" },
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
      className={`filmstrip-thumb ${isCurrent ? "active" : ""} ${borderClass}`}
      onClick={onClick}
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
        <div className="filmstrip-thumb-placeholder" />
      )}
      <div className="filmstrip-thumb-overlay">
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
  );
}
