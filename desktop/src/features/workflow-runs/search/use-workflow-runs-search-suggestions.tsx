import { ilike } from "@tanstack/db";
import { useLiveQuery } from "@tanstack/react-db";

import { filterPrefix } from "@/components/search";
import { tagsCollection, workflowsCollection } from "@/db/collections";
import { WorkflowRunFilter, WorkflowRunSearchSuggestion } from "./types";

const defaultSuggestions: WorkflowRunSearchSuggestion[] = [
  { id: "workflow-search-prefix", type: "prefix" as const, text: filterPrefix("workflow") },
  { id: "tag-search-prefix", type: "prefix" as const, text: filterPrefix("tag") },
];
type SuggestionQueryResultItem = {
  type: "workflowId" | "tag";
  text: string;
  sort_id: number;
};

/**
 * Builds the complete set of values that can be inserted into the workflow-run
 * search field. The live query keeps suggestions current as the local DB syncs.
 *
 * Workflow filters are mutually exclusive - when one is already present, workflow
 * suggestions are suppressed so a second workflow token cannot be added from the bar.
 */
export function useWorkflowRunsSearchSuggestions({
  filterValue,
  limit,
  activeFilters,
}: {
  filterValue: string;
  limit: number;
  activeFilters: WorkflowRunFilter[];
}) {
  const hasWorkflowFilter = activeFilters.some((filter) => filter.entity === "workflow");

  const { data, isLoading, isError, status } = useLiveQuery({
    query: (q) => {
      const workflowRows = q
        .from({ workflow: workflowsCollection })
        .select(({ workflow }) => ({
          type: "workflowId" as const,
          text: workflow.name,
          sort_id: workflow.id,
        }))
        .where(({ workflow }) => ilike(workflow.name, `${filterValue}%`))
        // Unique numeric keys — `created_at` ties can destabilize live orderBy.
        .orderBy(({ workflow }) => workflow.id, "asc")
        .limit(limit);

      const tagsRows = q
        .from({ tag: tagsCollection })
        .select(({ tag }) => ({
          type: "tag" as const,
          text: tag.value,
          sort_id: tag.id,
        }))
        .where(({ tag }) => ilike(tag.value, `${filterValue}%`))
        .orderBy(({ tag }) => tag.id, "asc")
        .limit(limit);

      return q.unionAll(workflowRows, tagsRows).orderBy(({ sort_id }) => sort_id);
    },
  });

  const suggestions = createSuggestions(data, hasWorkflowFilter);

  return { suggestions, isLoading, isError, status };
}

function createSuggestions(
  rows: SuggestionQueryResultItem[],
  hasWorkflowFilter: boolean,
): WorkflowRunSearchSuggestion[] {
  const workflowNames = hasWorkflowFilter
    ? []
    : rows.filter((row) => row.type === "workflowId").map((row) => row.text);
  const tagValues = rows.filter((row) => row.type === "tag").map((row) => row.text);

  const prefixes = hasWorkflowFilter
    ? defaultSuggestions.filter((suggestion) => suggestion.id !== "workflow-search-prefix")
    : defaultSuggestions;

  return [
    ...prefixes,
    ...[...new Set(workflowNames)]
      .sort((left, right) => left.localeCompare(right))
      .map((workflowName) => ({
        id: `workflow:${workflowName}`,
        type: "token" as const,
        text: `${filterPrefix("workflow")}${workflowName}`,
        value: { entity: "workflow" as const, id: workflowName },
      })),
    ...[...new Set(tagValues)]
      .sort((left, right) => left.localeCompare(right))
      .map((tagValue) => ({
        id: `tag:${tagValue}`,
        type: "token" as const,
        text: `${filterPrefix("tag")}${tagValue}`,
        value: { entity: "tag" as const, value: tagValue },
      })),
  ];
}
