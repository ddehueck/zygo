import { useLiveQuery } from "@tanstack/react-db";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import type { WorkflowRun } from "../bindings";
import { Cell, Column, Row, Table, TableBody, TableHeader } from "../components/Table";
import { Icons } from "../components/icons";
import { MainContentLayout } from "../components/layout/MainContentLayout";
import { useDuration } from "../hooks/use-duration";
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

const columns = [
  { id: "run", name: "Workflow run" },
  { id: "jobs", name: "Jobs" },
  { id: "duration", name: "Duration" },
] as const;

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
          <Table
            aria-label="Workflow runs"
            onRowAction={(key) =>
              navigate({
                to: "/runs/$workflowRunId",
                params: { workflowRunId: String(key) },
              })
            }
          >
            <TableHeader
              columns={columns}
              className="h-0 overflow-hidden [&>tr]:h-0 [&>tr>th]:h-0 [&>tr>th]:border-0 [&>tr>th]:p-0 [&>tr>th>*]:hidden"
            >
              {(column) => (
                <Column id={column.id} isRowHeader={column.id === "run"}>
                  {column.name}
                </Column>
              )}
            </TableHeader>
            <TableBody items={runs}>
              {(workflowRun) => <WorkflowRunRow id={workflowRun.id} workflowRun={workflowRun} />}
            </TableBody>
          </Table>
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

function WorkflowRunRow({ id, workflowRun }: { id: string; workflowRun: WorkflowRun }) {
  const totalJobs =
    workflowRun.active_job_count + workflowRun.succeeded_job_count + workflowRun.errored_job_count;
  const duration = useDuration({
    startedAt: workflowRun.started_at,
    completedAt: workflowRun.completed_at,
  });

  return (
    <Row
      id={id}
      textValue={`${workflowRun.id} ${statusLabel(workflowRun.status)} ${totalJobs} jobs`}
    >
      <Cell textValue={workflowRun.id}>
        <div className="flex min-w-64 items-center gap-3 py-1">
          <StatusIcon status={workflowRun.status} />
          <div className="min-w-0">
            <p className="truncate font-mono text-base font-semibold tracking-tight text-app-foreground">
              {workflowRun.workflow_id}{" "}
              <span className="text-app font-normal -foreground-muted">
                ({shortRunId(workflowRun.id)})
              </span>
            </p>
          </div>
        </div>
      </Cell>
      <Cell textValue={`${totalJobs} total jobs`}>
        <JobCountsBadge
          activeJobCount={workflowRun.active_job_count}
          succeededJobCount={workflowRun.succeeded_job_count}
          erroredJobCount={workflowRun.errored_job_count}
        />
      </Cell>
      <Cell textValue={duration ?? ""}>
        <div className="py-1">
          <p className="font-mono text-sm text-app-foreground">{duration ?? "—"}</p>
        </div>
      </Cell>
    </Row>
  );
}

function StatusIcon({ status }: { status: string }) {
  return (
    <span
      role="img"
      aria-label={statusLabel(status)}
      className="flex size-5 shrink-0 items-center justify-center"
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
