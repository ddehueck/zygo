import { BasicIndex, createCollection, localOnlyCollectionOptions } from "@tanstack/db";
import type { DataReference, JobRun, Tag, WorkflowRun } from "../bindings";

export const workflowRunsCollection = createCollection(
  localOnlyCollectionOptions<WorkflowRun, string>({
    id: "workflow_runs",
    getKey: (item) => item.id,
    defaultIndexType: BasicIndex,
    autoIndex: "eager",
  }),
);

export const jobRunsCollection = createCollection(
  localOnlyCollectionOptions<JobRun, string>({
    id: "job_runs",
    getKey: (item) => item.id,
    defaultIndexType: BasicIndex,
    autoIndex: "eager",
  }),
);

export const dataReferencesCollection = createCollection(
  localOnlyCollectionOptions<DataReference, string>({
    id: "data_references",
    getKey: (item) => item.id,
    defaultIndexType: BasicIndex,
    autoIndex: "eager",
  }),
);

export const tagsCollection = createCollection(
  localOnlyCollectionOptions<Tag, string>({
    id: "tags",
    getKey: (item) => item.id,
    defaultIndexType: BasicIndex,
    autoIndex: "eager",
  }),
);
