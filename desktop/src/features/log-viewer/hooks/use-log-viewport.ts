import { useCallback, useLayoutEffect, useRef, type RefObject } from "react";
import type { Virtualizer } from "@tanstack/react-virtual";

import { LOG_EDGE_THRESHOLD } from "../constants";
import { useLogViewportData } from "./use-log-viewport-data";
import { useLogVirtualizer } from "./use-log-virtualizer";

/**
 * Composes log data, end-anchored virtualization, and load-older fetching.
 * Prepend stability and follow-on-append are owned by the virtualizer.
 */
export function useLogViewport(workflowRunId: number) {
  const data = useLogViewportData(workflowRunId);
  const {
    logs,
    hasPreviousPage,
    isFetchingPreviousPage,
    fetchPreviousPage,
    isSearching,
    isRunActive,
  } = data;

  const showStartBoundary = logs.length > 0 && !hasPreviousPage;
  const showEndBoundary = logs.length > 0;
  const shouldFollow = isRunActive && !isSearching;

  const { scrollRef, virtualizer } = useLogVirtualizer(logs, {
    followOnAppend: shouldFollow,
    showStartBoundary,
    showEndBoundary,
  });
  const didScrollToEnd = useScrollToEndOnMount(logs.length, scrollRef, virtualizer, shouldFollow);

  const loadOlder = useCallback(() => {
    if (!hasPreviousPage || isFetchingPreviousPage) return;

    void fetchPreviousPage({ cancelRefetch: false, throwOnError: false }).catch(() => undefined);
  }, [fetchPreviousPage, hasPreviousPage, isFetchingPreviousPage]);

  const onScroll = useCallback(() => {
    if (!scrollRef.current) return;
    // While pinning to the live tail on mount, ignore top-edge fetches until settled.
    if (shouldFollow && !didScrollToEnd.current) return;

    // Content shorter than the viewport is "at end", not a deliberate scroll to older history.
    const atTop = scrollRef.current.scrollTop <= LOG_EDGE_THRESHOLD && !virtualizer.isAtEnd();
    if (atTop) loadOlder();
  }, [didScrollToEnd, loadOlder, scrollRef, shouldFollow, virtualizer]);

  const isFollowing = shouldFollow && virtualizer.isAtEnd();

  return {
    ...data,
    scrollRef,
    virtualizer,
    isFollowing,
    showStartBoundary,
    onScroll,
  };
}

/** Retry scrollToEnd until the flex scrollport is measurable and pinned. */
function useScrollToEndOnMount(
  logCount: number,
  scrollRef: RefObject<HTMLDivElement | null>,
  virtualizer: Virtualizer<HTMLDivElement, Element>,
  enabled: boolean,
) {
  const didScrollToEnd = useRef(false);

  useLayoutEffect(() => {
    if (!enabled || didScrollToEnd.current || logCount === 0) return;

    let cancelled = false;
    let frame = 0;

    const tryScroll = () => {
      if (cancelled) return;

      const element = scrollRef.current;
      if (!element || element.clientHeight === 0) {
        frame = requestAnimationFrame(tryScroll);
        return;
      }

      virtualizer.scrollToEnd();

      if (virtualizer.isAtEnd()) {
        didScrollToEnd.current = true;
        return;
      }

      frame = requestAnimationFrame(tryScroll);
    };

    tryScroll();

    return () => {
      cancelled = true;
      cancelAnimationFrame(frame);
    };
  }, [enabled, logCount, scrollRef, virtualizer]);

  return didScrollToEnd;
}
