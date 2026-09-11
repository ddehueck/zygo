import { BasicIndex, createCollection, localOnlyCollectionOptions } from "@tanstack/db";
import type { Log } from "@/bindings";
import { syncCollectionOptions } from "./sync-collection";

export const workflowsCollection = createCollection({
  ...syncCollectionOptions("workflow"),
  defaultIndexType: BasicIndex,
  autoIndex: "eager",
});

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

// Session-owned normalized rows. Reading this collection never performs IO.
export const logsCollection = createCollection({
  ...localOnlyCollectionOptions<Log>({ id: "logs", getKey: (log) => log.id }),
  gcTime: Infinity,
  startSync: true,
  defaultIndexType: BasicIndex,
  autoIndex: "eager",
});
