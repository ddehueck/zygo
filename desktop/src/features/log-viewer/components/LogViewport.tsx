import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import type { Log } from "@/bindings";
import { Text } from "@/components/Text";
import { LOG_OVERSCAN, LOG_ROW_ESTIMATED_HEIGHT } from "../constants";
import { useLogViewerScroll } from "../hooks/use-log-viewer-scroll";
import { LogRow } from "./LogRow";
import { LogViewerStatus } from "./LogViewerStatus";
import { LogViewerToolbar } from "./LogViewerToolbar";

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

  const virtualizer = useVirtualizer({
    count: logs.length,
    estimateSize: () => LOG_ROW_ESTIMATED_HEIGHT,
    getItemKey: (index) => logs[index]?.id ?? index,
    getScrollElement: () => scrollRef.current,
    overscan: LOG_OVERSCAN,
  });

  const { isFollowing, onScroll, scrollToLatest, setFollowing } = useLogViewerScroll({
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
      className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-app-border bg-app-bg-elevated"
    >
      <LogViewerToolbar
        isFollowing={isFollowing}
        onFollowChange={setFollowing}
        onJumpToLatest={scrollToLatest}
      />
      <div
        ref={scrollRef}
        onScroll={onScroll}
        className="relative min-h-0 flex-1 overflow-auto overscroll-contain"
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
          className="log-viewer-selectable relative w-full"
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
                <LogRow log={log} />
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
