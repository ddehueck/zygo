import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import type { Log } from "@/bindings";
import { LOG_EDGE_THRESHOLD, LOG_OVERSCAN, LOG_ROW_ESTIMATED_HEIGHT } from "../constants";

export function useLogVirtualizer(logs: Log[]) {
  const scrollRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: logs.length,
    estimateSize: () => LOG_ROW_ESTIMATED_HEIGHT,
    getItemKey: (index) => logs[index]?.id ?? index,
    getScrollElement: () => scrollRef.current,
    overscan: LOG_OVERSCAN,
    anchorTo: "end",
    followOnAppend: true,
    scrollEndThreshold: LOG_EDGE_THRESHOLD,
  });

  return { scrollRef, virtualizer };
}
