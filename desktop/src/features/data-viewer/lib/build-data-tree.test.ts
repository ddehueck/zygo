import { expect, test } from "vitest";

import type { JobRun, TauriDataReference } from "@/bindings";

import {
  buildDataTree,
  collectExpandedKeysUpToDepth,
  findDataTreeNode,
  flattenDataTree,
  type DataTreeNode,
} from "./build-data-tree";
import { DataTreeIndex } from "./data-tree-index";

const mocktree: DataTreeNode[] = [
  {
    id: 1,
    children: [{ id: 2, children: [{ id: 3, children: [] }] }],
  },
];

test("buildDataTree builds a tree", () => {
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

  expect(buildDataTree(index)).toEqual([{ id: 1, children: [{ id: 2, children: [] }] }]);
});

test("flattenDataTree walks pre-order", () => {
  expect(flattenDataTree(mocktree).map((node) => node.id)).toEqual([1, 2, 3]);
});

test("findDataTreeNode returns a nested node by id", () => {
  expect(findDataTreeNode(mocktree, 3)?.id).toBe(3);
});

test("collectExpandedKeysUpToDepth expands ancestors through maxDepth", () => {
  expect([...collectExpandedKeysUpToDepth(mocktree, 2)]).toEqual([1]);
});
