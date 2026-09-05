import { createTransaction } from "@tanstack/db";
import { QueryClient } from "@tanstack/query-core";
import { commands } from "../bindings";
import { upsertCollectionItem } from "./collection-helpers";
import { jobRuns, tags, workflowRuns } from "./collections";

const INITIAL_WORKFLOW_RUN_LIMIT = 500;

export const queryClient = new QueryClient();

const snapshotQueryOptions = {
  queryKey: ["snapshot"],
  queryFn: async () => {
    const result = await commands.loadData({
      cursor: null,
      limit: INITIAL_WORKFLOW_RUN_LIMIT,
    });

    if (result.status === "error") {
      throw new Error(result.error.message);
    }

    return result.data;
  },
  staleTime: Infinity,
  gcTime: Infinity,
};

export async function loadSnapshot() {
  const snapshot = await queryClient.ensureQueryData(snapshotQueryOptions);

  const hydrationTransaction = createTransaction({
    autoCommit: false,
    mutationFn: async ({ transaction }) => {
      workflowRuns.utils.acceptMutations(transaction);
      jobRuns.utils.acceptMutations(transaction);
      tags.utils.acceptMutations(transaction);
    },
  });

  hydrationTransaction.mutate(() => {
    for (const workflowRun of snapshot.workflow_runs) {
      upsertCollectionItem(workflowRuns, workflowRun);
    }

    for (const jobRun of snapshot.job_runs) {
      upsertCollectionItem(jobRuns, jobRun);
    }

    for (const tag of snapshot.tags) {
      upsertCollectionItem(tags, tag);
    }
  });

  await hydrationTransaction.commit();
  return snapshot;
}
