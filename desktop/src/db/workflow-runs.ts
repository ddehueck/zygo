import { collectionOptions } from "@tanstack/db";
import { QueryClient } from "@tanstack/query-core";
import { parseLoadSubsetOptions, queryCollectionOptions } from "@tanstack/query-db-collection";
import { commands } from "../bindings";
import { syncEntityRefreshOptions, tdb } from "./shared";

const DEFAULT_LIMIT = 100;

// Define a stable collection descriptor that loads data using TanStack Query
const options = collectionOptions("workflow_runs", (client) =>
  queryCollectionOptions({
    id: "workflow_runs",
    queryKey: ["workflow_runs"],
    queryClient: client.requireDependency<QueryClient>("queryClient"),
    ...syncEntityRefreshOptions,
    queryFn: async (ctx) => {
      const options = parseLoadSubsetOptions(ctx.meta?.loadSubsetOptions);

      const result = await commands.listWorkflowRuns({
        cursor: null,
        limit: options.limit ?? DEFAULT_LIMIT,
      });

      if (result.status === "error") {
        console.log(result.error);
        throw new Error(result.error.message);
      }

      return result.data;
    },
    // https://tanstack.com/db/latest/docs/collections/query-collection#selecting-rows-from-wrapped-responses
    select: (response) => response.runs,
    getKey: (item) => item.id,
  }),
);

export const workflowRuns = tdb.collection(options);
