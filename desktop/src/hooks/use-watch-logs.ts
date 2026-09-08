import { eq, queryOnce } from "@tanstack/db";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { commands } from "@/bindings";
import { logsCollection } from "@/db/collections";
import { last } from "@/lib/arrays";
import { isPositiveInteger } from "@/lib/integer";
import { invariant } from "@/utils";
import { useKeepCollectionAlive } from "./use-keep-collection-alive";

const PAGE_SIZE = 1000;
const POLL_INTERVAL = 1000;

type LogWatchProgress = {
  afterId: number;
  hasMore: boolean;
};

export function useWatchLogs({
  workflowRunId,
  enabled = true,
}: {
  workflowRunId: number;
  enabled?: boolean;
}) {
  const queryClient = useQueryClient();
  const queryKey = ["log-watch", workflowRunId] as const;

  // Ensures data we write won't just disappear when this is running in the background.
  useKeepCollectionAlive(logsCollection, enabled);

  return useQuery<LogWatchProgress>({
    queryKey,
    enabled,
    networkMode: "always",
    refetchIntervalInBackground: true,
    refetchInterval: (query) =>
      query.state.status !== "error" && query.state.data?.hasMore ? 1 : POLL_INTERVAL,
    queryFn: async ({ signal }): Promise<LogWatchProgress> => {
      invariant(isPositiveInteger(workflowRunId), "Invalid workflow run ID");

      const progress = queryClient.getQueryData<LogWatchProgress>(queryKey);
      const cursor = progress?.afterId ?? (await getMaxLogId(workflowRunId));
      signal.throwIfAborted();

      const result = await commands.queryLogs({
        workflow_run_id: workflowRunId,
        after_id: cursor,
        offset: 0,
        limit: PAGE_SIZE,
      });

      // Tauri command calls cannot be canceled, so discard results if needed.
      signal.throwIfAborted();
      if (result.status === "error") throw new Error(result.error.message);

      const nextCursor = last(result.data)?.id ?? cursor;
      invariant(
        result.data.length === 0 || nextCursor > cursor,
        "Log polling cursor must advance of be without data",
      );

      if (result.data.length > 0) logsCollection.utils.writeUpsert(result.data);

      return { afterId: nextCursor, hasMore: result.data.length === PAGE_SIZE };
    },
  });
}

async function getMaxLogId(workflowRunId: number): Promise<number> {
  const logs = await queryOnce((q) =>
    q
      .from({ log: logsCollection })
      .where(({ log }) => eq(log.workflow_run_id, workflowRunId))
      .orderBy(({ log }) => log.id, "desc")
      .limit(1),
  );

  return logs[0]?.id ?? 0;
}
