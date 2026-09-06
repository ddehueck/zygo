import { createTransaction } from "@tanstack/db";
import { QueryClient } from "@tanstack/query-core";
import { commands } from "../bindings";
import { upsertCollectionItem } from "./collection-helpers";
import {
  dataReferencesCollection,
  jobRunsCollection,
  tagsCollection,
  workflowRunsCollection,
} from "./collections";

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
      workflowRunsCollection.utils.acceptMutations(transaction);
      jobRunsCollection.utils.acceptMutations(transaction);
      tagsCollection.utils.acceptMutations(transaction);
      dataReferencesCollection.utils.acceptMutations(transaction);
    },
  });

  hydrationTransaction.mutate(() => {
    for (const workflowRun of snapshot.workflow_runs) {
      upsertCollectionItem(workflowRunsCollection, workflowRun);
    }

    for (const jobRun of snapshot.job_runs) {
      upsertCollectionItem(jobRunsCollection, jobRun);
    }

    for (const tag of snapshot.tags) {
      upsertCollectionItem(tagsCollection, tag);
    }

    for (const reference of snapshot.data_references) {
      upsertCollectionItem(dataReferencesCollection, reference);
    }
  });

  await hydrationTransaction.commit();
  return snapshot;
}
