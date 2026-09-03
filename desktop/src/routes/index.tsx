import { useLiveQuery } from "@tanstack/react-db";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import type { WorkflowRunSummary } from "../bindings";
import { Cell, Column, Row, Table, TableBody, TableHeader } from "../components/Table";
import { Icons } from "../components/icons";
import { MainContentLayout } from "../components/layout/MainContentLayout";
import { useDuration } from "../hooks/use-duration";
import { workflowRuns } from "../db/workflow-run-summaries";

export const Route = createFileRoute("/")({
  beforeLoad: () => ({
    breadcrumb: {
      label: "Workflow runs",
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
    <MainContentLayout titleContent={null}>
      <main className="flex w-full flex-col gap-6 py-10">
        <header className="flex items-end justify-between gap-4 px-6">
          <div>
            <p className="mb-2 text-sm font-medium uppercase tracking-wide text-app-foreground-muted">
              Runs
            </p>
            <h1 className="text-3xl font-semibold tracking-tight text-app-foreground">
              Workflow runs
            </h1>
            <p className="mt-2 text-app-foreground-muted">
              Monitor workflow status and job progress at a glance.
            </p>
          </div>
          {!isLoading && !isError && (
            <div className="shrink-0 text-right">
              <p className="text-2xl font-semibold tabular-nums text-app-foreground">
                {runs.length}
              </p>
              <p className="text-xs font-medium uppercase tracking-wide text-app-foreground-muted">
                Total runs
              </p>
            </div>
          )}
        </header>

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
              {(workflowRun) => (
                <WorkflowRunRow
                  id={workflowRun.workflow_run_id}
                  workflowRun={workflowRun}
                />
              )}
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
    </MainContentLayout>
  );
}

function WorkflowRunRow({
  id,
  workflowRun,
}: {
  id: string;
  workflowRun: WorkflowRunSummary;
}) {
  const totalJobs =
    workflowRun.active_job_count + workflowRun.succeeded_job_count + workflowRun.errored_job_count;
  const duration = useDuration({
    startedAt: workflowRun.started_at,
    completedAt: workflowRun.completed_at,
  });

  return (
    <Row
      id={id}
      textValue={`${workflowRun.workflow_run_id} ${statusLabel(workflowRun.status)} ${totalJobs} jobs`}
    >
      <Cell textValue={workflowRun.workflow_run_id}>
        <div className="flex min-w-64 items-center gap-3 py-1">
          <StatusIcon status={workflowRun.status} />
          <div className="min-w-0">
            <p
              className="truncate font-mono text-base font-semibold tracking-tight text-app-foreground"
            >
              …{shortRunId(workflowRun.workflow_run_id)}
            </p>

          </div>
        </div>
      </Cell>
      <Cell textValue={`${totalJobs} total jobs`}>
        <JobCounts
          running={workflowRun.active_job_count}
          completed={workflowRun.succeeded_job_count}
          errored={workflowRun.errored_job_count}
          total={totalJobs}
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

function JobCounts({
  running,
  completed,
  errored,
  total,
}: {
  running: number;
  completed: number;
  errored: number;
  total: number;
}) {
  return (
    <div
      className="flex items-center gap-3 whitespace-nowrap py-1"
      aria-label={`${total} total jobs`}
    >
      <JobCount kind="running" value={running} label="running jobs" />
      <JobCount kind="completed" value={completed} label="completed jobs" />
      <JobCount kind="errored" value={errored} label="errored jobs" />

    </div>
  );
}

function JobCount({
  kind,
  value,
  label,
}: {
  kind: "running" | "completed" | "errored";
  value: number;
  label: string;
}) {
  return (
    <span
      className="inline-flex items-center gap-1 text-xs font-medium text-app-foreground"
      aria-label={`${value} ${label}`}
    >
      <JobGlyph kind={kind} />
      <span className="tabular-nums">{value}</span>
    </span>
  );
}

function JobGlyph({ kind }: { kind: "running" | "completed" | "errored" }) {
  const iconProps = { "aria-hidden": true, className: "size-3.5", strokeWidth: 2.25 } as const;

  switch (kind) {
    case "running":
      return <Icons.InProgress {...iconProps} />;
    case "completed":
      return <Icons.Completed {...iconProps} />;
    case "errored":
      return <Icons.Errored {...iconProps} />;
  }
}

function shortRunId(workflowRunId: string): string {
  return workflowRunId.slice(-4);
}

function statusLabel(status: string): string {
  return status.replace(/_/g, " ");
}

