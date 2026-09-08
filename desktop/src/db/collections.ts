import { BasicIndex, createCollection, LoadSubsetOptions } from "@tanstack/db";

import { parseLoadSubsetOptions, queryCollectionOptions } from "@tanstack/query-db-collection";
import { commands, type Log } from "@/bindings";
import { syncCollectionOptions } from "./sync-collection";
import { queryClient } from "./query-client";
import { invariant } from "@/utils";
import { err, ok, Result } from "@/lib/result";

export const workflowRunsCollection = createCollection({
  ...syncCollectionOptions("workflow_run"),
  defaultIndexType: BasicIndex,
  autoIndex: "eager",
});

export const jobRunsCollection = createCollection({
  ...syncCollectionOptions("job_run"),
  defaultIndexType: BasicIndex,
  autoIndex: "eager",
});

export const dataReferencesCollection = createCollection({
  ...syncCollectionOptions("data_reference"),
  defaultIndexType: BasicIndex,
  autoIndex: "eager",
});

export const tagsCollection = createCollection({
  ...syncCollectionOptions("tag"),
  defaultIndexType: BasicIndex,
  autoIndex: "eager",
});

export const logsCollection = createCollection(
  queryCollectionOptions<Log>({
    id: "logs",
    queryKey: (options) =>
      options.where ? ["logs", JSON.stringify(parseOptionsForLogs(options))] : ["logs"],
    syncMode: "on-demand",
    queryClient,
    getKey: (log) => log.id,
    queryFn: async (context) => {
      const subsetOptions = context.meta?.loadSubsetOptions;
      invariant(!!subsetOptions, "Logs collection must be accessed with params set.");

      const parsed = parseOptionsForLogs(subsetOptions);
      if (!parsed.success) throw new Error(parsed.error);

      const result = await commands.queryLogs({
        workflow_run_id: parsed.data.workflow_run_id,
        limit: parsed.data.limit,
        offset: parsed.data.offset,
        after_id: null,
      });

      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
  }),
);

function parseOptionsForLogs(
  options: LoadSubsetOptions,
): Result<{ workflow_run_id: number; offset: number; limit: number }, string> {
  const { offset, limit } = options;
  const { filters } = parseLoadSubsetOptions(options);

  const equalFilter = filters.find(
    (filter) =>
      filter.field.length === 1 &&
      filter.field[0] === "workflow_run_id" &&
      filter.operator === "eq",
  );

  if (!equalFilter || typeof equalFilter.value !== "number") {
    return err("Could not find any workflow run id in query");
  }

  return ok({
    offset: offset ?? 0,
    limit: limit ?? 1000,
    workflow_run_id: equalFilter.value,
  });
}
