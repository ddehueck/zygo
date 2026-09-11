import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useEffect, useRef, useState, type DragEvent } from "react";

type UseFileDropOptions = {
  /** Called with absolute filesystem paths when files are dropped on the window. */
  onDrop?: (paths: string[]) => void;
  /** When false, the webview listener is not attached. Defaults to true. */
  enabled?: boolean;
};

/**
 * Tracks OS-level file drag-and-drop on the Tauri webview.
 *
 * Prefer this over browser DnD for picking local paths: Tauri delivers real
 * filesystem paths, while browser drop events do not in the desktop shell.
 */
export function useFileDrop({ onDrop, enabled = true }: UseFileDropOptions = {}) {
  const [isDragging, setIsDragging] = useState(false);
  const onDropRef = useRef(onDrop);
  onDropRef.current = onDrop;

  useEffect(() => {
    if (!enabled) {
      setIsDragging(false);
      return;
    }

    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void (async () => {
      unlisten = await getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type === "over") {
          setIsDragging(true);
          return;
        }

        if (event.payload.type === "leave" || event.payload.type === "drop") {
          setIsDragging(false);
        }

        if (event.payload.type === "drop") {
          onDropRef.current?.(event.payload.paths);
        }
      });

      if (cancelled) unlisten();
    })();

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [enabled]);

  return {
    isDragging,
    /** Spread onto a drop target to keep browser DnD from navigating the page. */
    dropTargetProps: {
      onDragOver(event: DragEvent<HTMLElement>) {
        event.preventDefault();
        setIsDragging(true);
      },
      onDragLeave() {
        setIsDragging(false);
      },
      onDrop(event: DragEvent<HTMLElement>) {
        event.preventDefault();
        setIsDragging(false);
      },
    },
  };
}
