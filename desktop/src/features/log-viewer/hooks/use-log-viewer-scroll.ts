import { useCallback, useLayoutEffect, useRef, useState, type RefObject } from "react";
import type { Virtualizer } from "@tanstack/react-virtual";

import type { Log } from "@/bindings";
import { LOG_BOTTOM_THRESHOLD } from "../constants";

type LogVirtualizer = Virtualizer<HTMLDivElement, Element>;

type LoadSnapshot = {
  rowCount: number;
  scrollOffset: number;
  totalSize: number;
};

export function useLogViewerScroll({
  scrollRef,
  virtualizer,
  logs,
  hasNextPage,
  isFetchingNextPage,
  fetchNextPage,
}: {
  scrollRef: RefObject<HTMLDivElement | null>;
  virtualizer: LogVirtualizer;
  logs: Log[];
  hasNextPage: boolean;
  isFetchingNextPage: boolean;
  fetchNextPage: () => Promise<void>;
}) {
  const [isFollowing, setIsFollowing] = useState(true);
  const didSetInitialPosition = useRef(false);
  const previousNewestId = useRef<number | undefined>(undefined);
  const loadSnapshot = useRef<LoadSnapshot | null>(null);

  const scrollToLatest = useCallback(() => {
    if (logs.length === 0) return;
    setIsFollowing(true);
    virtualizer.scrollToIndex(logs.length - 1, { align: "end" });
  }, [logs.length, virtualizer]);

  const onScroll = useCallback(() => {
    const element = scrollRef.current;
    if (!element) return;

    const distanceFromBottom = element.scrollHeight - element.scrollTop - element.clientHeight;
    setIsFollowing(distanceFromBottom <= LOG_BOTTOM_THRESHOLD);

    if (
      element.scrollTop <= LOG_BOTTOM_THRESHOLD &&
      hasNextPage &&
      !isFetchingNextPage &&
      loadSnapshot.current === null
    ) {
      loadSnapshot.current = {
        rowCount: logs.length,
        scrollOffset: element.scrollTop,
        totalSize: virtualizer.getTotalSize(),
      };
      void fetchNextPage();
    }
  }, [fetchNextPage, hasNextPage, isFetchingNextPage, logs.length, scrollRef, virtualizer]);

  useLayoutEffect(() => {
    if (logs.length === 0) return;

    if (!didSetInitialPosition.current) {
      didSetInitialPosition.current = true;
      previousNewestId.current = logs[logs.length - 1]?.id;
      virtualizer.scrollToIndex(logs.length - 1, { align: "end" });
      return;
    }

    const snapshot = loadSnapshot.current;
    if (snapshot && logs.length > snapshot.rowCount) {
      const addedHeight = virtualizer.getTotalSize() - snapshot.totalSize;
      virtualizer.scrollToOffset(snapshot.scrollOffset + addedHeight);
      loadSnapshot.current = null;
    }

    const newestId = logs[logs.length - 1]?.id;
    if (newestId !== previousNewestId.current) {
      previousNewestId.current = newestId;
      if (isFollowing) virtualizer.scrollToIndex(logs.length - 1, { align: "end" });
    }
  }, [isFollowing, logs, virtualizer]);

  const setFollowing = useCallback(
    (shouldFollow: boolean) => {
      if (shouldFollow) scrollToLatest();
      else setIsFollowing(false);
    },
    [scrollToLatest],
  );

  return { isFollowing, onScroll, scrollToLatest, setFollowing };
}
