import { useQuery } from "@tanstack/react-query";
import { queryClient } from "@/db/query-client";
import { last } from "@/lib/arrays";
import { fetchLogsPage } from "../api/fetch-logs-page";
import { LOG_PAGE_SIZE, NEW_LOGS_CHECK_INTERVAL_MS } from "../constants";

type Progress = { afterId: number };

export function useWatchLogs({
  workflowRunId,
  enabled = true,
  /** When false, fetch once to hydrate then stop (completed runs). */
  watch = true,
  initialAfterId,
}: {
  workflowRunId: number;
  enabled?: boolean;
  watch?: boolean;
  initialAfterId?: number;
}) {
  const queryKey = ["log-watch", workflowRunId] as const;

  return useQuery({
    queryKey,
    enabled,
    networkMode: "always",
    refetchIntervalInBackground: true,
    refetchInterval: (query) =>
      !watch || query.state.status === "error" ? false : NEW_LOGS_CHECK_INTERVAL_MS,
    queryFn: async (): Promise<Progress> => {
      // `after_id` present (including 0) pages ASC from that bound. Absent pages
      // DESC from the newest. Never default a missing cursor to 0 — that would
      // drain the entire history on every cold start.
      let afterId: number | undefined =
        queryClient.getQueryData<Progress>(queryKey)?.afterId ?? initialAfterId;

      if (afterId === undefined) {
        const page = await fetchLogsPage({
          workflow_run_id: workflowRunId,
          limit: LOG_PAGE_SIZE,
        });
        return { afterId: last(page.logs)?.id ?? 0 };
      }

      // Known cursor: drain anything newer inside this query instead of
      // refetchInterval: 1 (which freezes the UI with per-ms React updates).
      let hasMore = false;
      do {
        const page = await fetchLogsPage({
          workflow_run_id: workflowRunId,
          after_id: afterId,
          limit: 1000,
        });
        afterId = last(page.logs)?.id ?? afterId;
        hasMore = page.has_more;
        if (hasMore) {
          await new Promise<void>((resolve) => setTimeout(resolve, 0));
        }
      } while (hasMore);

      return { afterId };
    },
  });
}
