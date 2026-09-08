import { eq, ilike, like, not } from "@tanstack/db";
import { useDebouncedValue } from "@tanstack/react-pacer";
import { useQuery } from "@tanstack/react-query";
import { useLiveQuery } from "@tanstack/react-db";

import { logsCollection } from "@/db/collections";
import { last } from "@/lib/arrays";
import { fetchLogsPage } from "../api/fetch-logs-page";
import { useInfiniteLogPages } from "../api/use-infinite-log-pages";
import { LOG_SEARCH_DEBOUNCE_MS, LOG_SEARCH_PAGE_SIZE } from "../constants";
import { useLogSearchContext } from "../search/LogSearchContext";
import { useWatchLogs } from "./use-watch-logs";

const SYSTEM_LOG_PREFIX = "ZYGO_IPC=";

export function useLogViewportData(workflowRunId: number) {
  const { cleanedQuery } = useLogSearchContext();
  const [debouncedSearch] = useDebouncedValue(cleanedQuery, { wait: LOG_SEARCH_DEBOUNCE_MS });

  const pages = useInfiniteLogPages(workflowRunId);
  const initialPage = last(pages.data?.pages ?? []);

  const watcher = useWatchLogs({
    workflowRunId,
    enabled: pages.data !== undefined,
    initialAfterId: last(initialPage?.logs ?? [])?.id,
  });

  useQuery({
    queryKey: ["logs-search", workflowRunId, debouncedSearch],
    enabled: debouncedSearch.length > 0,
    staleTime: 0,
    networkMode: "always",
    queryFn: () =>
      fetchLogsPage({
        workflow_run_id: workflowRunId,
        limit: LOG_SEARCH_PAGE_SIZE,
        search: debouncedSearch,
      }),
  });

  const liveQuery = useLiveQuery(
    (q) => {
      let query = q
        .from({ log: logsCollection })
        .where(({ log }) => eq(log.workflow_run_id, workflowRunId))
        .where(({ log }) => not(like(log.content, `${SYSTEM_LOG_PREFIX}%`)));

      if (cleanedQuery) {
        query = query.where(({ log }) => ilike(log.content, `%${cleanedQuery}%`));
      }

      return query.orderBy(({ log }) => log.id, "asc");
    },
    [workflowRunId, cleanedQuery],
  );

  return {
    isLoading: pages.isPending,
    isError: pages.isError && pages.data === undefined,
    logs: liveQuery.data,
    isSearching: cleanedQuery.length > 0,
    hasPreviousPage: pages.hasPreviousPage,
    hasNewer: watcher.data?.hasMore ?? false,
    isFetchingPreviousPage: pages.isFetchingPreviousPage,
    fetchPreviousPage: pages.fetchPreviousPage,
    error: pages.error ?? watcher.error,
  };
}
