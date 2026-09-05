import { useHotkey } from "@tanstack/react-hotkeys";
import { useRef } from "react";

export function useWorkflowRunListHotkeys() {
  const listRef = useRef<HTMLDivElement>(null);

  const focusList = () => {
    const list = listRef.current;
    if (list && !list.contains(document.activeElement)) {
      list.focus();
    }
  };

  useHotkey("ArrowUp", focusList);
  useHotkey("ArrowDown", focusList);

  return listRef;
}
