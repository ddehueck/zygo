import { BasicIndex, createCollection, localOnlyCollectionOptions } from "@tanstack/db";
import type { DataReference, JobRun, Tag, WorkflowRun } from "../bindings";

export const workflowRunsCollection = createCollection(
  localOnlyCollectionOptions<WorkflowRun, number>({
    id: "workflow_runs",
    getKey: (item) => item.id,
    defaultIndexType: BasicIndex,
    autoIndex: "eager",
  }),
);

export const jobRunsCollection = createCollection(
  localOnlyCollectionOptions<JobRun, number>({
    id: "job_runs",
    getKey: (item) => item.id,
    defaultIndexType: BasicIndex,
    autoIndex: "eager",
  }),
);

export const dataReferencesCollection = createCollection(
  localOnlyCollectionOptions<DataReference, number>({
    id: "data_references",
    getKey: (item) => item.id,
    defaultIndexType: BasicIndex,
    autoIndex: "eager",
  }),
);

export const tagsCollection = createCollection(
  localOnlyCollectionOptions<Tag, number>({
    id: "tags",
    getKey: (item) => item.id,
    defaultIndexType: BasicIndex,
    autoIndex: "eager",
  }),
);
