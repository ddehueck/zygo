import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import type { Log } from "@/bindings";
import { boolToInt } from "@/lib/integer";
import { LOG_EDGE_THRESHOLD, LOG_OVERSCAN, LOG_ROW_ESTIMATED_HEIGHT } from "../constants";

export type LogViewportRow =
  | { type: "boundary"; edge: "start" | "end" }
  | { type: "log"; log: Log };

export function getLogViewportRow(
  index: number,
  logs: Log[],
  showStartBoundary: boolean,
): LogViewportRow {
  if (showStartBoundary && index === 0) return { type: "boundary", edge: "start" };

  const logIndex = showStartBoundary ? index - 1 : index;
  if (logIndex >= 0 && logIndex < logs.length) {
    return { type: "log", log: logs[logIndex]! };
  }

  return { type: "boundary", edge: "end" };
}

export function useLogVirtualizer(
  logs: Log[],
  {
    followOnAppend,
    showStartBoundary,
    showEndBoundary,
  }: {
    followOnAppend: boolean;
    showStartBoundary: boolean;
    showEndBoundary: boolean;
  },
) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const startOffset = boolToInt(showStartBoundary);
  const count = logs.length + startOffset + boolToInt(showEndBoundary);

  const virtualizer = useVirtualizer({
    count,
    estimateSize: () => LOG_ROW_ESTIMATED_HEIGHT,
    getItemKey: (index) => {
      if (showStartBoundary && index === 0) return "log-boundary-start";
      const logIndex = index - startOffset;
      if (logIndex >= 0 && logIndex < logs.length) return logs[logIndex]?.id ?? index;
      // Tie the end sentinel to the latest log so followOnAppend sees appends.
      // A stable key would leave the list's last key unchanged when rows are
      // inserted before the boundary, which suppresses auto-follow.
      return `log-boundary-end:${logs[logs.length - 1]?.id ?? "empty"}`;
    },
    getScrollElement: () => scrollRef.current,
    overscan: LOG_OVERSCAN,
    anchorTo: "end",
    followOnAppend,
    scrollEndThreshold: LOG_EDGE_THRESHOLD,
    // React 19: flushSync during measure/ref attach warns when already rendering.
    useFlushSync: false,
  });

  return { scrollRef, virtualizer };
}
