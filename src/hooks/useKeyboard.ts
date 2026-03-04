import { useEffect, useRef } from "react";
import type { PickStatus } from "./useCull";

interface KeyboardActions {
  next: () => void;
  prev: () => void;
  setPickStatus: (status: PickStatus) => Promise<void>;
  setRating: (rating: number) => Promise<void>;
  openExport: () => void;
  closeExport: () => void;
  toggleView: () => void;
}

export function useKeyboard(actions: KeyboardActions) {
  const actionsRef = useRef(actions);
  actionsRef.current = actions;

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const a = actionsRef.current;

      switch (e.key) {
        // Navigation pure
        case "ArrowRight":
          e.preventDefault();
          a.next();
          break;
        case "ArrowLeft":
          e.preventDefault();
          a.prev();
          break;
        case "ArrowUp":
          e.preventDefault();
          a.prev();
          break;
        case "ArrowDown":
          e.preventDefault();
          a.next();
          break;

        // Statut
        case "d":
        case "D":
          a.setPickStatus("pick");
          break;
        case "q":
        case "Q":
          a.setPickStatus("reject");
          break;
        case "s":
        case "S":
          a.setPickStatus("unrated");
          break;

        // Notes
        case "1":
          a.setRating(1);
          break;
        case "2":
          a.setRating(2);
          break;
        case "3":
          a.setRating(3);
          break;
        case "4":
          a.setRating(4);
          break;
        case "5":
          a.setRating(5);
          break;
        case "0":
          a.setRating(0);
          break;

        // Vue
        case "Tab":
          e.preventDefault();
          a.toggleView();
          break;

        // Export
        case "e":
        case "E":
          a.openExport();
          break;
        case "Escape":
          a.closeExport();
          break;
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);
}
