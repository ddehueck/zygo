import { BasicIndex, createCollection, localOnlyCollectionOptions } from "@tanstack/db";
import type { JobRun, Tag, WorkflowRun } from "../bindings";

export const workflowRuns = createCollection(
  localOnlyCollectionOptions<WorkflowRun, string>({
    id: "workflow_runs",
    getKey: (item) => item.id,
    defaultIndexType: BasicIndex,
    autoIndex: "eager",
  }),
);

export const jobRuns = createCollection(
  localOnlyCollectionOptions<JobRun, string>({
    id: "job_runs",
    getKey: (item) => item.id,
    defaultIndexType: BasicIndex,
    autoIndex: "eager",
  }),
);

export const tags = createCollection(
  localOnlyCollectionOptions<Tag, string>({
    id: "tags",
    getKey: (item) => item.id,
    defaultIndexType: BasicIndex,
    autoIndex: "eager",
  }),
);
