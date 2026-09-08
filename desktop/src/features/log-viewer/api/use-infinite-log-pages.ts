import { useInfiniteQuery } from "@tanstack/react-query";

import { LOG_PAGE_SIZE } from "../constants";
import { fetchLogsPage } from "./fetch-logs-page";

export function useInfiniteLogPages(workflowRunId: number) {
  return useInfiniteQuery({
    queryKey: ["logs-pages", workflowRunId],
    staleTime: 0,
    // Drop the view's history on close; exact page requests remain shared.
    gcTime: 0,
    networkMode: "always",
    refetchOnMount: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
    initialPageParam: null as number | null,
    queryFn: ({ pageParam }) =>
      fetchLogsPage({
        workflow_run_id: workflowRunId,
        before_id: pageParam ?? undefined,
        limit: LOG_PAGE_SIZE,
      }),
    getPreviousPageParam: (firstPage) => (firstPage.has_more ? firstPage.logs[0]?.id : undefined),
    getNextPageParam: () => undefined,
  });
}
