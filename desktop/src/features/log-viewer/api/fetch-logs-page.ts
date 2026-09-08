import { z } from "zod";

import { commands, type QueryLogsRequest, type QueryLogsResponse } from "@/bindings";
import { logsCollection } from "@/db/collections";
import { queryClient } from "@/db/query-client";

type LogPage = Pick<QueryLogsResponse, "logs" | "has_more">;

const logPageRequestSchema = z
  .object({
    workflow_run_id: z.int().positive(),
    limit: z.int().min(1).max(1000),
    after_id: z.int().nonnegative().nullish(),
    before_id: z.int().positive().nullish(),
    search: z.string().nullish(),
    job_run_id: z.int().positive().nullish(),
  })
  .refine((request) => request.before_id == null || request.before_id > (request.after_id ?? 0), {
    path: ["before_id"],
    message: "before_id must be greater than after_id (or 0)",
  }) satisfies z.ZodType<QueryLogsRequest>;

export async function fetchLogsPage(request: QueryLogsRequest): Promise<LogPage> {
  const parsed = logPageRequestSchema.parse(request);
  const after_id = parsed.after_id ?? null;
  const before_id = parsed.before_id ?? null;
  const search = parsed.search?.trim() || null;
  const job_run_id = parsed.job_run_id ?? null;

  const page = await queryClient.query({
    queryKey: ["logs-page", { ...parsed, after_id, before_id, search, job_run_id }],
    staleTime: (query) => logsPageStaleTime(before_id, search, job_run_id, query.state.data),
    gcTime: 5 * 60_000,
    networkMode: "always",
    queryFn: async () => {
      const result = await commands.queryLogs({
        workflow_run_id: parsed.workflow_run_id,
        limit: parsed.limit,
        after_id,
        before_id,
        search,
        job_run_id,
      });
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
  });

  // Also rehydrate on request-cache hits after row loss.
  const missing = page.logs.filter((log) => !logsCollection.has(log.id));
  if (missing.length) logsCollection.insert(missing);
  return { logs: page.logs, has_more: page.has_more };
}

// Cache forever only if we know that the entire range has been written on the server.
// i.e. don't cache forever if we are watching a range at the log tail
export function logsPageStaleTime(
  beforeId: number | null,
  search: string | null,
  jobRunId: number | null,
  page: Pick<QueryLogsResponse, "global_watermark_id"> | undefined,
): number {
  if (search !== null || jobRunId !== null || beforeId === null) return 0;
  return beforeId - 1 <= (page?.global_watermark_id ?? -1) ? Infinity : 0;
}
