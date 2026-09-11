import { eq, materialize } from "@tanstack/db";
import { useLiveQuery } from "@tanstack/react-db";
import {
  tagsCollection,
  workflowRunsCollection,
  workflowsCollection,
} from "../../../db/collections";

export type WorkflowRunListData = ReturnType<typeof useWorkflowRunsListData>["data"];

export function useWorkflowRunsListData() {
  return useLiveQuery({
    query: (q) =>
      q
        .from({ workflowRun: workflowRunsCollection })
        .leftJoin(
          { workflow: workflowsCollection },
          ({ workflowRun, workflow }) => eq(workflowRun.workflow_id, workflow.id),
        )
        .select(({ workflowRun, workflow }) => ({
          workflowRun,
          workflow,
          tags: materialize(
            q
              .from({ tag: tagsCollection })
              .where(({ tag }) => eq(workflowRun.id, tag.workflow_run_id))

              .orderBy(({ tag }) => tag.value, "asc")
              .orderBy(({ tag }) => tag.id, "asc"),
          ),
        }))
        .orderBy(({ workflowRun }) => workflowRun.id, "desc"),
  });
}
