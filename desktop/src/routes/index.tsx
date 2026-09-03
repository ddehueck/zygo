import { useLiveQuery } from "@tanstack/react-db";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import type { WorkflowRun } from "../bindings";
import { GridList, GridListItem } from "../components/GridList";
import { Icons } from "../components/icons";
import { useDuration } from "../hooks/use-duration";
import { sum } from "../lib/math";
import { workflowRuns } from "../db/collections";
import { JobCountsBadge } from "../features/workflow-runs/components/JobCountsBadge";
import { shortRunId } from "../features/workflow-runs/lib/id";
import { BreadcrumbHeaderLayout } from "../components/layout/BreadcrumbHeaderLayout";
import { useHotkey } from "@tanstack/react-hotkeys";
import { useRef } from "react";
import { WorkflowRunList } from "../features/workflow-runs/components/WorkflowRunList";

export const Route = createFileRoute("/")({
  beforeLoad: () => ({
    breadcrumb: {
      label: "Workflow Runs",
      link: "/",
    },
  }),
  component: IndexRoute,
});

function IndexRoute() {
  const {
    data: runs,
    isLoading,
    isError,
    status,
  } = useLiveQuery({
    query: (q) =>
      q
        .from({ workflowRun: workflowRuns })
        .orderBy(({ workflowRun }) => workflowRun.created_at, "desc"),
  });

  return (
    <BreadcrumbHeaderLayout>
      <main className="flex w-full flex-col gap-6">
        {isLoading && <p className="px-6 text-app-foreground-muted">Loading workflow runs…</p>}
        {isError && (
          <p
            className="mx-6 rounded-md border border-app-danger/30 bg-app-danger/10 p-4 text-app-danger"
            role="alert"
          >
            Unable to load workflow runs (status: {status}).
          </p>
        )}

        {runs.length > 0 ? (
          <WorkflowRunList runs={runs} />
        ) : (
          !isLoading &&
          !isError && (
            <p className="mx-6 rounded-md border border-dashed border-app-border p-8 text-center text-app-foreground-muted">
              No workflow runs loaded.
            </p>
          )
        )}
      </main>
    </BreadcrumbHeaderLayout>
  );
}
