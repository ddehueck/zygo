import { useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import type { Log } from "@/bindings";
import { scrollAreaClassName } from "@/components/ScrollArea";
import { Text } from "@/components/Text";
import { cn } from "@/components/utils";
import { LOG_OVERSCAN, LOG_ROW_ESTIMATED_HEIGHT } from "../constants";
import { useLogViewerScroll } from "../hooks/use-log-viewer-scroll";
import { LogRow } from "./LogRow";
import { LogViewerStatus } from "./LogViewerStatus";
import { LogViewportHeader } from "./LogViewportHeader";

export function LogViewport({
  logs,
  hasNextPage,
  isFetchingNextPage,
  fetchNextPage,
  watchError,
}: {
  logs: Log[];
  hasNextPage: boolean;
  isFetchingNextPage: boolean;
  fetchNextPage: () => Promise<void>;
  watchError: unknown;
}) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [showDate, setShowDate] = useState(true);
  const [showJobId, setShowJobId] = useState(true);

  const virtualizer = useVirtualizer({
    count: logs.length,
    estimateSize: () => LOG_ROW_ESTIMATED_HEIGHT,
    getItemKey: (index) => logs[index]?.id ?? index,
    getScrollElement: () => scrollRef.current,
    overscan: LOG_OVERSCAN,
  });

  const { isFollowing, onScroll } = useLogViewerScroll({
    scrollRef,
    virtualizer,
    logs,
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
  });

  return (
    <section
      aria-label="Workflow run logs"
      className="flex min-h-0 w-full flex-1 flex-col overflow-hidden bg-app-bg-elevated"
    >
      <LogViewportHeader
        showDate={showDate}
        showJobId={showJobId}
        onToggleDate={() => setShowDate((visible) => !visible)}
        onToggleJobId={() => setShowJobId((visible) => !visible)}
      />
      <div
        ref={scrollRef}
        onScroll={onScroll}
        className={cn(scrollAreaClassName, "relative overflow-y-auto overscroll-none")}
      >
        {isFetchingNextPage && (
          <div className="sticky top-0 z-10 flex justify-center py-2" aria-live="polite">
            <Text
              size="small"
              variant="muted"
              className="rounded-full border border-app-border bg-app-bg-elevated px-3 py-1 shadow-sm"
            >
              Loading older logs…
            </Text>
          </div>
        )}
        <div
          className="highlightable relative w-full"
          style={{ height: virtualizer.getTotalSize() }}
        >
          {virtualizer.getVirtualItems().map((virtualRow) => {
            const log = logs[virtualRow.index];
            if (!log) return null;

            return (
              <div
                key={virtualRow.key}
                ref={virtualizer.measureElement}
                data-index={virtualRow.index}
                className="absolute top-0 left-0 w-full"
                style={{ transform: `translateY(${virtualRow.start}px)` }}
              >
                <LogRow log={log} showDate={showDate} showJobId={showJobId} />
              </div>
            );
          })}
        </div>
      </div>
      <LogViewerStatus
        rowCount={logs.length}
        isFollowing={isFollowing}
        isFetchingOlder={isFetchingNextPage}
        hasOlder={hasNextPage}
        watchError={watchError}
      />
    </section>
  );
}
