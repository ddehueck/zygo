import { concat, ilike, or } from "@tanstack/db";
import { useLiveQuery } from "@tanstack/react-db";

import { tags, workflowRuns } from "@/db/collections";
import { WorkflowRunSearchSuggestion } from "./types";

const defaultSuggestions: WorkflowRunSearchSuggestion[] = [
  { id: "workflow-search-prefix", type: "prefix" as const, text: "@workflow:" },
  { id: "tag-search-prefix", type: "prefix" as const, text: "@tag:" },
];

type SuggestionQueryResultItem = {
  type: "workflowId" | "tag";
  text: string;
  created_at: string;
};

/**
 * Builds the complete set of values that can be inserted into the workflow-run
 * search field. The live query keeps suggestions current as the local DB syncs.
 */
export function useWorkflowRunsSearchSuggestions({
  filterValue,
  limit,
}: {
  filterValue: string;
  limit: number;
}) {
  const { data, isLoading, isError, status } = useLiveQuery({
    query: (q) => {
      const runIdRows = q
        .from({ run: workflowRuns })
        .select(({ run }) => ({
          type: "workflowId" as const,
          text: run.workflow_id,
          created_at: run.created_at,
        }))
        .where(({ run }) => ilike(run.workflow_id, `${filterValue}%`))
        .orderBy(({ run }) => run.created_at, "asc")
        .limit(limit);

      const tagsRows = q
        .from({ tag: tags })
        .select(({ tag }) => ({
          type: "tag" as const,
          text: concat(tag.key, ":", tag.value),
          created_at: tag.created_at,
        }))
        .where(({ tag }) =>
          or(ilike(tag.key, `${filterValue}%`), ilike(tag.value, `${filterValue}%`)),
        )
        .orderBy(({ tag }) => tag.created_at, "asc")
        .limit(limit);

      return q.unionAll(runIdRows, tagsRows).orderBy(({ created_at }) => created_at);
    },
  });

  const suggestions = createSuggestions(data);

  return { suggestions, isLoading, isError, status };
}

function createSuggestions(rows: SuggestionQueryResultItem[]): WorkflowRunSearchSuggestion[] {
  const workflowIds = rows.filter((row) => row.type === "workflowId").map((row) => row.text);
  const tagValues = rows.filter((row) => row.type === "tag").map((row) => row.text);

  return [
    ...defaultSuggestions,
    ...[...workflowIds]
      .sort((left, right) => left.localeCompare(right))
      .map((workflowId) => ({
        id: `workflow:${workflowId}`,
        type: "token" as const,
        text: `@workflow:${workflowId}`,
      })),
    ...[...tagValues]
      .sort((left, right) => left.localeCompare(right))
      .map((tagValue) => ({
        id: `tag:${tagValue}`,
        type: "token" as const,
        text: `@tag:${tagValue}`,
      })),
  ];
}
