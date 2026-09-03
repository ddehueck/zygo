import { useLiveQuery } from "@tanstack/react-db";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import type { WorkflowRun } from "../bindings";
import { GridList, GridListItem } from "../components/GridList";
import { Icons } from "../components/icons";
import { useDuration } from "../hooks/use-duration";
import { sum } from "../lib/math";
import { workflowRuns } from "../db/workflow-runs";
import { JobCountsBadge } from "../features/workflow-runs/components/JobCountsBadge";
import { shortRunId } from "../features/workflow-runs/lib/id";
import { BreadcrumbHeaderLayout } from "../components/layout/BreadcrumbHeaderLayout";

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
  const navigate = useNavigate();

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
          <GridList
            aria-label="Workflow runs"
            items={runs}
            onAction={(key) =>
              navigate({
                to: "/runs/$workflowRunId",
                params: { workflowRunId: String(key) },
              })
            }
          >
            {(workflowRun) => <WorkflowRunItem id={workflowRun.id} workflowRun={workflowRun} />}
          </GridList>
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

function WorkflowRunItem({ id, workflowRun }: { id: string; workflowRun: WorkflowRun }) {
  const totalJobs = sum([
    workflowRun.active_job_count,
    workflowRun.succeeded_job_count,
    workflowRun.errored_job_count,
  ]);

  const duration = useDuration({
    startedAt: workflowRun.started_at,
    completedAt: workflowRun.completed_at,
  });

  return (
    <GridListItem
      id={id}
      textValue={`${workflowRun.id} ${statusLabel(workflowRun.status)} ${totalJobs} jobs`}
    >
      <div className="grid min-w-0 grid-cols-[minmax(0,1fr)_auto_auto] items-center gap-6">
        <div className="flex min-w-0 items-center gap-3 py-1 pl-1.5">
          <StatusIcon status={workflowRun.status} />
          <div className="min-w-0">
            <p className="truncate font-mono text-base font-semibold tracking-tight text-app-foreground">
              <span className="text-app -foreground-muted font-normal">
                ({shortRunId(workflowRun.id)})
              </span>{" "}
              {workflowRun.workflow_id}
            </p>
          </div>
        </div>
        <div className="flex justify-end">
          <JobCountsBadge
            activeJobCount={workflowRun.active_job_count}
            succeededJobCount={workflowRun.succeeded_job_count}
            erroredJobCount={workflowRun.errored_job_count}
          />
        </div>
        <div className="flex justify-end py-1 pr-1.5">
          <p className="font-mono text-sm text-app-foreground">{duration ?? "—"}</p>
        </div>
      </div>
    </GridListItem>
  );
}

function StatusIcon({ status }: { status: string }) {
  return (
    <span
      role="img"
      aria-label={statusLabel(status)}
      className="flex size-3 shrink-0 items-center justify-center"
    >
      <StatusGlyph status={status} className="size-5" />
    </span>
  );
}

function StatusGlyph({ status, className }: { status: string; className: string }) {
  const iconProps = { "aria-hidden": true, className, strokeWidth: 2.25 } as const;

  switch (status) {
    case "succeeded":
      return <Icons.Completed {...iconProps} />;
    case "running":
      return <Icons.InProgress {...iconProps} />;
    case "failed":
    case "errored":
      return <Icons.Errored {...iconProps} />;
    default:
      return null;
  }
}

function statusLabel(status: string): string {
  return status.replace(/_/g, " ");
}
