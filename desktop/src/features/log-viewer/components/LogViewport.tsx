import { useState } from "react";

import { scrollAreaClassName } from "@/components/ScrollArea";
import { Description, Text } from "@/components/Text";
import { cn } from "@/components/utils";
import { useLogViewport } from "../hooks/use-log-viewport";
import { LogRow } from "./LogRow";
import { LogViewerStatus } from "./LogViewerStatus";
import { LogViewportHeader } from "./LogViewportHeader";

export function LogViewport({ workflowRunId }: { workflowRunId: number }) {
  const {
    logs,
    hasPreviousPage,
    isFetchingPreviousPage,
    error,
    isLoading,
    isError,
    scrollRef,
    virtualizer,
    isFollowing,
    onScroll,
    loadOlder,
  } = useLogViewport(workflowRunId);

  const [showDate, setShowDate] = useState(true);
  const [showJobId, setShowJobId] = useState(true);

  if (isLoading) {
    return (
      <div className="flex min-h-64 flex-1 items-center justify-center">
        <Text variant="muted">Loading workflow logs…</Text>
      </div>
    );
  }

  if (isError) {
    return (
      <div className="flex min-h-64 flex-1 items-center justify-center">
        <Text role="alert" size="small" variant="danger">
          Unable to load workflow logs.
        </Text>
      </div>
    );
  }

  if (logs.length === 0 && !hasPreviousPage) {
    return (
      <div className="flex min-h-64 flex-1 items-center justify-center">
        {error ? (
          <Text role="alert" size="small" variant="danger">
            Unable to load logs.
          </Text>
        ) : (
          <Description>No visible logs recorded for this workflow run.</Description>
        )}
      </div>
    );
  }

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
      {hasPreviousPage && (
        <button
          type="button"
          disabled={isFetchingPreviousPage}
          className="shrink-0 py-2"
          onClick={loadOlder}
        >
          <Text size="small" variant="muted">
            {isFetchingPreviousPage ? "Loading older logs…" : "Load older logs"}
          </Text>
        </button>
      )}
      <div
        ref={scrollRef}
        onScroll={onScroll}
        className={cn(
          scrollAreaClassName,
          "relative min-h-0 flex-1 overflow-y-auto overscroll-none",
        )}
        style={{ overflowAnchor: "none" }}
      >
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
        isFetchingPreviousPage={isFetchingPreviousPage}
        hasPreviousPage={hasPreviousPage}
        error={error}
      />
    </section>
  );
}
