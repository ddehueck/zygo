import { eq } from "@tanstack/db";
import { useLiveInfiniteQuery } from "@tanstack/react-db";

import { logsCollection } from "@/db/collections";
import { useWatchLogs } from "@/hooks/use-watch-logs";
import { LOG_PAGE_SIZE } from "../constants";

// Hide system-level IPC logs from the user-facing log view.
const SYSTEM_LOG_PREFIX = "ZYGO_IPC=";

export function useLogViewerData(workflowRunId: number) {
  const watcher = useWatchLogs({ workflowRunId });
  const query = useLiveInfiniteQuery(
    (q) =>
      q
        .from({ log: logsCollection })
        .where(({ log }) => eq(log.workflow_run_id, workflowRunId))
        .orderBy(({ log }) => log.id, "desc"),
    { pageSize: LOG_PAGE_SIZE },
  );

  return {
    ...query,
    logs: [...query.data].filter((log) => !log.content.startsWith(SYSTEM_LOG_PREFIX)).reverse(),
    watchError: watcher.error,
  };
}
