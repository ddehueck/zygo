import { eq, ilike, like, not } from "@tanstack/db";
import { useDebouncedValue } from "@tanstack/react-pacer";
import { useQuery } from "@tanstack/react-query";
import { useLiveQuery } from "@tanstack/react-db";

import { logsCollection, workflowRunsCollection } from "@/db/collections";
import { isTerminalWorkflowRunStatus } from "@/features/workflow-runs/components/statuses";
import { last } from "@/lib/arrays";
import { fetchLogsPage } from "../api/fetch-logs-page";
import { useInfiniteLogPages } from "../api/use-infinite-log-pages";
import { LOG_SEARCH_DEBOUNCE_MS, LOG_SEARCH_PAGE_SIZE } from "../constants";
import { useLogSearchContext } from "../search/LogSearchContext";
import { useWatchLogs } from "./use-watch-logs";

const SYSTEM_LOG_PREFIX = "ZYGO_IPC=";

export function useLogViewportData(workflowRunId: number) {
  const { contentQuery, jobRunId, jobRunPublicId, isFiltering } = useLogSearchContext();
  const [debouncedContentQuery] = useDebouncedValue(contentQuery, {
    wait: LOG_SEARCH_DEBOUNCE_MS,
  });
  const [debouncedJobRunId] = useDebouncedValue(jobRunId, {
    wait: LOG_SEARCH_DEBOUNCE_MS,
  });
  const shouldFetchSearch = debouncedContentQuery.length > 0 || debouncedJobRunId != null;

  const runQuery = useLiveQuery({
    query: (q) =>
      q
        .from({ workflowRun: workflowRunsCollection })
        .where(({ workflowRun }) => eq(workflowRun.id, workflowRunId))
        .findOne(),
  });
  const isRunActive = runQuery.data != null && !isTerminalWorkflowRunStatus(runQuery.data.status);

  const pages = useInfiniteLogPages(workflowRunId);
  const initialPage = last(pages.data?.pages ?? []);

  const watcher = useWatchLogs({
    workflowRunId,
    enabled: pages.data !== undefined && isRunActive,
    initialAfterId: last(initialPage?.logs ?? [])?.id,
  });

  useQuery({
    queryKey: ["logs-search", workflowRunId, debouncedContentQuery, debouncedJobRunId],
    enabled: shouldFetchSearch,
    staleTime: 0,
    networkMode: "always",
    queryFn: () =>
      fetchLogsPage({
        workflow_run_id: workflowRunId,
        limit: LOG_SEARCH_PAGE_SIZE,
        search: debouncedContentQuery || null,
        job_run_id: debouncedJobRunId,
      }),
  });

  const liveQuery = useLiveQuery({
    query: (q) => {
      let query = q
        .from({ log: logsCollection })
        .where(({ log }) => eq(log.workflow_run_id, workflowRunId))
        .where(({ log }) => not(like(log.content, `${SYSTEM_LOG_PREFIX}%`)));

      if (jobRunPublicId) {
        query = query.where(({ log }) => eq(log.job_run_id, jobRunPublicId));
      }

      if (contentQuery) {
        query = query.where(({ log }) => ilike(log.content, `%${contentQuery}%`));
      }

      return query.orderBy(({ log }) => log.id, "asc");
    },
  });

  return {
    isLoading: pages.isPending,
    isError: pages.isError && pages.data === undefined,
    logs: liveQuery.data,
    isSearching: isFiltering,
    isRunActive,
    hasPreviousPage: pages.hasPreviousPage,
    isFetchingPreviousPage: pages.isFetchingPreviousPage,
    fetchPreviousPage: pages.fetchPreviousPage,
    error: pages.error ?? watcher.error,
  };
}
