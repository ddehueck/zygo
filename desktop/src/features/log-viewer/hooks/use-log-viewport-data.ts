import { eq, like, not } from "@tanstack/db";
import { useLiveQuery } from "@tanstack/react-db";

import { logsCollection } from "@/db/collections";
import { useInfiniteLogPages } from "../api/use-infinite-log-pages";
import { useWatchLogs } from "./use-watch-logs";
import { last } from "@/lib/arrays";

const SYSTEM_LOG_PREFIX = "ZYGO_IPC=";

export function useLogViewportData(workflowRunId: number) {
  const pages = useInfiniteLogPages(workflowRunId);
  const initialPage = last(pages.data?.pages ?? []);

  const watcher = useWatchLogs({
    workflowRunId,
    enabled: pages.data !== undefined,
    initialAfterId: last(initialPage?.logs ?? [])?.id,
  });

  const query = useLiveQuery(
    (q) =>
      q
        .from({ log: logsCollection })
        .where(({ log }) => eq(log.workflow_run_id, workflowRunId))
        .where(({ log }) => not(like(log.content, `${SYSTEM_LOG_PREFIX}%`)))
        .orderBy(({ log }) => log.id, "asc"),
    [workflowRunId],
  );

  return {
    isLoading: pages.isPending,
    isError: pages.isError && pages.data === undefined,
    logs: query.data,
    hasPreviousPage: pages.hasPreviousPage,
    hasNewer: watcher.data?.hasMore ?? false,
    isFetchingPreviousPage: pages.isFetchingPreviousPage,
    fetchPreviousPage: pages.fetchPreviousPage,
    error: pages.error ?? watcher.error,
  };
}
