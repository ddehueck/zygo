import { eq, materialize } from "@tanstack/db";
import { useLiveQuery } from "@tanstack/react-db";
import { tags, workflowRuns } from "../../../db/collections";

export type WorkflowRunListData = ReturnType<typeof useWorkflowRunsListData>["data"];

export function useWorkflowRunsListData() {
  return useLiveQuery({
    query: (q) =>
      q
        .from({ workflowRun: workflowRuns })
        .select(({ workflowRun }) => ({
          workflowRun,
          tags: materialize(
            q
              .from({ tag: tags })
              .where(({ tag }) => eq(workflowRun.id, tag.workflow_run_id))
              .orderBy(({ tag }) => tag.created_at, "asc"),
          ),
        }))
        .orderBy(({ workflowRun }) => workflowRun.created_at, "desc"),
  });
}
