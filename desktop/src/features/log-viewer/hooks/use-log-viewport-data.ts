import { eq, ilike, like, not } from "@tanstack/db";
import { useQuery } from "@tanstack/react-query";
import { useLiveQuery } from "@tanstack/react-db";

import { logsCollection } from "@/db/collections";
import { last } from "@/lib/arrays";
import { fetchLogsPage } from "../api/fetch-logs-page";
import { useInfiniteLogPages } from "../api/use-infinite-log-pages";
import { LOG_SEARCH_PAGE_SIZE } from "../constants";
import { useLogSearchContext } from "../search/LogSearchContext";
import { useWatchLogs } from "./use-watch-logs";

const SYSTEM_LOG_PREFIX = "ZYGO_IPC=";

export function useLogViewportData(workflowRunId: number) {
  const { query: searchQuery } = useLogSearchContext();
  const search = searchQuery.trim();

  const pages = useInfiniteLogPages(workflowRunId);
  const initialPage = last(pages.data?.pages ?? []);

  const watcher = useWatchLogs({
    workflowRunId,
    enabled: pages.data !== undefined,
    initialAfterId: last(initialPage?.logs ?? [])?.id,
  });

  useQuery({
    queryKey: ["logs-search", workflowRunId, search],
    enabled: search.length > 0,
    staleTime: 0,
    networkMode: "always",
    queryFn: () =>
      fetchLogsPage({
        workflow_run_id: workflowRunId,
        limit: LOG_SEARCH_PAGE_SIZE,
        search,
      }),
  });

  const liveQuery = useLiveQuery(
    (q) => {
      let query = q
        .from({ log: logsCollection })
        .where(({ log }) => eq(log.workflow_run_id, workflowRunId))
        .where(({ log }) => not(like(log.content, `${SYSTEM_LOG_PREFIX}%`)));

      if (search) {
        query = query.where(({ log }) => ilike(log.content, `%${search}%`));
      }

      return query.orderBy(({ log }) => log.id, "asc");
    },
    [workflowRunId, search],
  );

  return {
    isLoading: pages.isPending,
    isError: pages.isError && pages.data === undefined,
    logs: liveQuery.data,
    isSearching: search.length > 0,
    hasPreviousPage: pages.hasPreviousPage,
    hasNewer: watcher.data?.hasMore ?? false,
    isFetchingPreviousPage: pages.isFetchingPreviousPage,
    fetchPreviousPage: pages.fetchPreviousPage,
    error: pages.error ?? watcher.error,
  };
}
