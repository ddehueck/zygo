import { eq, materialize } from "@tanstack/db";
import { useLiveQuery } from "@tanstack/react-db";

import type { Tag } from "@/bindings";
import { tags, workflowRuns } from "@/db/collections";
import { WorkflowRunSearchSuggestion } from "./types";

const defaultSuggestions: WorkflowRunSearchSuggestion[] = [
  { id: "workflow-search-prefix", type: "prefix" as const, text: "@workflow:" },
  { id: "tag-search-prefix", type: "prefix" as const, text: "@tag:" },
];

type WorkflowRunSearchSource = {
  workflowId: string;
  tags: Tag[];
};

/**
 * Builds the complete set of values that can be inserted into the workflow-run
 * search field. The live query keeps suggestions current as the local DB syncs.
 */
export function useWorkflowRunsSearchSuggestions() {
  const { data, isLoading, isError, status } = useLiveQuery({
    query: (q) =>
      q.from({ workflowRun: workflowRuns }).select(({ workflowRun }) => ({
        workflowId: workflowRun.workflow_id,
        tags: materialize(
          q
            .from({ tag: tags })
            .where(({ tag }) => eq(workflowRun.id, tag.workflow_run_id))
            .orderBy(({ tag }) => tag.created_at, "asc"),
        ),
      })),
  });

  const suggestions = createSuggestions(data);

  return { suggestions, isLoading, isError, status };
}

function createSuggestions(rows: WorkflowRunSearchSource[]): WorkflowRunSearchSuggestion[] {
  const workflowIds = new Set<string>();
  const tagValues = new Set<string>();

  for (const row of rows) {
    workflowIds.add(row.workflowId);
    for (const tag of row.tags) {
      tagValues.add(`${tag.key}:${tag.value}`);
    }
  }

  return [
    ...defaultSuggestions,
    ...[...workflowIds]
      .sort((left, right) => left.localeCompare(right))
      .map((workflowId) => ({
        id: `workflow:${workflowId}`,
        type: "token" as const,
        text: `@workflow:${workflowId}`,
        value: "workflow" as const,
      })),
    ...[...tagValues]
      .sort((left, right) => left.localeCompare(right))
      .map((tagValue) => ({
        id: `tag:${tagValue}`,
        type: "token" as const,
        text: `@tag:${tagValue}`,
        value: "tag" as const,
      })),
  ];
}
