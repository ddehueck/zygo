import { BasicIndex, createCollection } from "@tanstack/db";
import { syncCollectionOptions } from "./sync-collection";

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
