import { useQuery } from "@tanstack/react-query";
import { logsCollection } from "@/db/collections";
import { queryClient } from "@/db/query-client";
import { fetchLogsPage } from "../api/fetch-logs-page";
import { LOG_PAGE_SIZE, NEW_LOGS_CHECK_INTERVAL_MS } from "../constants";

type Progress = { afterId: number; hasMore: boolean };

export function useWatchLogs({
  workflowRunId,
  enabled = true,
  initialAfterId,
}: {
  workflowRunId: number;
  enabled?: boolean;
  initialAfterId?: number;
}) {
  const queryKey = ["log-watch", workflowRunId] as const;
  return useQuery({
    queryKey,
    enabled,
    networkMode: "always",
    refetchIntervalInBackground: true,
    refetchInterval: (query) =>
      query.state.status !== "error" && query.state.data?.hasMore ? 1 : NEW_LOGS_CHECK_INTERVAL_MS,
    queryFn: async (): Promise<Progress> => {
      const cached =
        queryClient.getQueryData<Progress>(queryKey) ??
        (initialAfterId === undefined ? undefined : { afterId: initialAfterId, hasMore: false });

      const previous =
        cached && (cached.afterId === 0 || logsCollection.has(cached.afterId)) ? cached : undefined;

      const page = await fetchLogsPage({
        workflow_run_id: workflowRunId,
        after_id: previous?.afterId,
        limit: previous ? 1000 : LOG_PAGE_SIZE,
      });

      return {
        afterId: page.logs[page.logs.length - 1]?.id ?? previous?.afterId ?? 0,
        hasMore: previous !== undefined && page.has_more,
      };
    },
  });
}
