import { expect, test } from "vitest";

import type { JobRun, TauriDataReference } from "@/bindings";

import { DataTreeIndex } from "./data-tree-index";

test("DataTreeIndex finds roots and child ids", () => {
  const references: TauriDataReference[] = [
    {
      id: 1,
      workflow_run_id: 1,
      source_job_run_id: null,
      uri: "uri://root",
      is_replay: false,
      created_at: "2024-01-01T00:00:00Z",
    },
    {
      id: 2,
      workflow_run_id: 1,
      source_job_run_id: 10,
      uri: "uri://child",
      is_replay: false,
      created_at: "2024-01-01T00:00:01Z",
    },
  ];
  const jobs: JobRun[] = [
    {
      id: 10,
      public_id: "job-10",
      workflow_run_id: 1,
      input_id: 1,
      job_id: "transform",
      status: "succeeded",
      duration_ms: null,
      retry_count: 0,
      created_at: "2024-01-01T00:00:00Z",
    },
  ];

  const index = new DataTreeIndex(references, jobs);

  expect(index.rootIds).toEqual([1]);
  expect(index.childIds(1)).toEqual([2]);
});
