import { collectionOptions } from "@tanstack/db";
import { QueryClient } from "@tanstack/query-core";
import { parseLoadSubsetOptions, queryCollectionOptions } from "@tanstack/query-db-collection";
import { useDbClient } from "@tanstack/react-db";
import { commands } from "../bindings";

const DEFAULT_LIMIT = 100;

// Define a stable collection descriptor that loads data using TanStack Query
const workflowRunsCollection = collectionOptions("workflow_runs", (client) =>
  queryCollectionOptions({
    id: "workflow_runs",
    queryKey: ["workflow_runs"],
    queryClient: client.requireDependency<QueryClient>("queryClient"),
    // CDC events are the live-update mechanism; refetching can apply an older
    // full snapshot after a newer CDC update.
    staleTime: Infinity,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
    queryFn: async (ctx) => {
      const options = parseLoadSubsetOptions(ctx.meta?.loadSubsetOptions);

      const result = await commands.listWorkflowRunSummaries({
        cursor: null,
        limit: options.limit ?? DEFAULT_LIMIT,
      });

      if (result.status === "error") {
        throw new Error(result.error);
      }

      return result.data;
    },
    // https://tanstack.com/db/latest/docs/collections/query-collection#selecting-rows-from-wrapped-responses
    select: (response) => response.summaries,
    getKey: (item) => item.workflow_run_id,
  }),
);

export function useWorkflowRunsCollection() {
  return useDbClient().collection(workflowRunsCollection);
}

export function useWorkflowRunsActions() {
  const collection = useWorkflowRunsCollection();
  // https://tanstack.com/db/latest/docs/collections/query-collection#direct-writes
  return {
    upsert: collection.utils.writeUpsert,
    delete: collection.utils.writeDelete,
  };
}
