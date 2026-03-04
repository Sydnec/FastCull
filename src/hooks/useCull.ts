import { useState, useCallback, useEffect, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface FileInfo {
  index: number;
  filename: string;
  size: number;
  status: "unrated" | "pick" | "reject";
  rating: number;
}

export type PickStatus = "unrated" | "pick" | "reject";
export type ExportMode = "move" | "copy" | "xmponly";

export interface ExportFilter {
  statuses: PickStatus[];
  min_rating: number;
}

export interface ViewFilter {
  showPick: boolean;
  showReject: boolean;
  showUnrated: boolean;
  minRating: number;
}

interface PrefetchProgress {
  cached: number[];
  total: number;
}

export interface ExportResult {
  exported_count: number;
  xmp_count: number;
  output_dir: string | null;
}

export interface CullState {
  files: FileInfo[];
  currentIndex: number;
  currentFile: FileInfo | null;
  currentStatus: PickStatus;
  currentRating: number;
  counts: { total: number; picks: number; rejects: number };
  statuses: Map<number, PickStatus>;
  ratings: Map<number, number>;
  filter: ViewFilter;
  setFilter: (filter: ViewFilter) => void;
  filteredIndices: number[];
  filteredPosition: number;
  prefetchCached: number[];
  isLoading: boolean;
  sessionId: number;
  openPaths: (paths: string[]) => Promise<void>;
  resetSession: () => void;
  navigate: (index: number) => void;
  next: () => void;
  prev: () => void;
  pick: () => void;
  reject: () => void;
  setPickStatus: (status: PickStatus) => Promise<void>;
  setRating: (rating: number) => Promise<void>;
  exportSelected: (mode: ExportMode, filter: ExportFilter) => Promise<ExportResult>;
}

export function useCull(): CullState {
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [statuses, setStatuses] = useState<Map<number, PickStatus>>(new Map());
  const [ratings, setRatings] = useState<Map<number, number>>(new Map());
  const [prefetchCached, setPrefetchCached] = useState<number[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [sessionId, setSessionId] = useState(0);
  const [filter, setFilter] = useState<ViewFilter>({
    showPick: true,
    showReject: true,
    showUnrated: true,
    minRating: 0,
  });

  // Refs pour accéder aux valeurs courantes dans les callbacks
  const filesRef = useRef(files);
  const currentIndexRef = useRef(currentIndex);
  filesRef.current = files;
  currentIndexRef.current = currentIndex;

  // Indices filtrés (memoized)
  const filteredIndices = useMemo(() => {
    const { showPick, showReject, showUnrated, minRating } = filter;
    if (showPick && showReject && showUnrated && minRating === 0) {
      return files.map((_, i) => i);
    }
    return files
      .map((_, i) => i)
      .filter((i) => {
        const status = statuses.get(i) ?? "unrated";
        const rating = ratings.get(i) ?? 0;
        const statusOk =
          (showPick && status === "pick") ||
          (showReject && status === "reject") ||
          (showUnrated && status === "unrated");
        return statusOk && rating >= minRating;
      });
  }, [files, statuses, ratings, filter]);

  const filteredIndicesRef = useRef(filteredIndices);
  filteredIndicesRef.current = filteredIndices;

  const filteredPosition = useMemo(() => {
    const idx = filteredIndices.indexOf(currentIndex);
    return idx >= 0 ? idx : -1;
  }, [filteredIndices, currentIndex]);

  // Écouter les événements prefetch_progress
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    listen<PrefetchProgress>("prefetch_progress", (event) => {
      setPrefetchCached(event.payload.cached);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  const openPaths = useCallback(async (paths: string[]) => {
    setIsLoading(true);
    try {
      const result = await invoke<FileInfo[]>("open_paths", { paths });
      setFiles(result);
      setCurrentIndex(0);
      setStatuses(new Map());
      setRatings(new Map());
      setPrefetchCached([]);
      setSessionId((s) => s + 1);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const resetSession = useCallback(() => {
    setFiles([]);
    setCurrentIndex(0);
    setStatuses(new Map());
    setRatings(new Map());
    setPrefetchCached([]);
    setFilter({ showPick: true, showReject: true, showUnrated: true, minRating: 0 });
  }, []);

  const navigate = useCallback((index: number) => {
    if (index < 0 || index >= filesRef.current.length) return;
    setCurrentIndex(index);
    invoke("navigate", { index }).catch((e) =>
      console.error("navigate error:", e),
    );
  }, []);

  // Navigation filtrée : next() saute au prochain index filtré
  const next = useCallback(() => {
    const current = currentIndexRef.current;
    const filtered = filteredIndicesRef.current;
    const nextIdx = filtered.find((i) => i > current);
    if (nextIdx !== undefined) {
      navigate(nextIdx);
    }
  }, [navigate]);

  // Navigation filtrée : prev() recule au précédent index filtré
  const prev = useCallback(() => {
    const current = currentIndexRef.current;
    const filtered = filteredIndicesRef.current;
    for (let j = filtered.length - 1; j >= 0; j--) {
      if (filtered[j] < current) {
        navigate(filtered[j]);
        return;
      }
    }
  }, [navigate]);

  const setPickStatus = useCallback(async (status: PickStatus) => {
    const idx = currentIndexRef.current;
    await invoke("set_pick_status", { index: idx, status });
    setStatuses((prev) => new Map(prev).set(idx, status));
  }, []);

  const setRating = useCallback(async (rating: number) => {
    const idx = currentIndexRef.current;
    await invoke("set_rating", { index: idx, rating });
    setRatings((prev) => new Map(prev).set(idx, rating));
  }, []);

  const pick = useCallback(async () => {
    await setPickStatus("pick");
    next();
  }, [setPickStatus, next]);

  const reject = useCallback(async () => {
    await setPickStatus("reject");
    next();
  }, [setPickStatus, next]);

  const exportSelected = useCallback(
    async (mode: ExportMode, filter: ExportFilter): Promise<ExportResult> => {
      return await invoke<ExportResult>("export_selected", { mode, filter });
    },
    [],
  );

  const currentFile = files[currentIndex] ?? null;
  const currentStatus = statuses.get(currentIndex) ?? "unrated";
  const currentRating = ratings.get(currentIndex) ?? 0;

  const counts = {
    total: files.length,
    picks: Array.from(statuses.values()).filter((s) => s === "pick").length,
    rejects: Array.from(statuses.values()).filter((s) => s === "reject").length,
  };

  return {
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
    filteredPosition,
    prefetchCached,
    isLoading,
    sessionId,
    openPaths,
    resetSession,
    navigate,
    next,
    prev,
    pick,
    reject,
    setPickStatus,
    setRating,
    exportSelected,
  };
}
