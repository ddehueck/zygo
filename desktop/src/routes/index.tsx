import { useLiveQuery } from "@tanstack/react-db";
import { useEffect, useState } from "react";
import { createFileRoute } from "@tanstack/react-router";
import type { WorkflowRunSummary } from "../bindings";
import { Button } from "../components/Button";
import { Cell, Column, Row, Table, TableBody, TableHeader } from "../components/Table";
import { workflowRuns } from "../db/workflow-run-summaries";
import { useTheme } from "../hooks/use-theme";

export const Route = createFileRoute("/")({
  component: IndexRoute,
});

const columns = [
  { id: "workflowRunId", name: "Workflow run" },
  { id: "status", name: "Status" },
  { id: "created", name: "Created" },
  { id: "started", name: "Started" },
  { id: "completed", name: "Completed" },
  { id: "duration", name: "Duration" },
  { id: "totalJobs", name: "Total jobs" },
  { id: "succeededJobs", name: "Succeeded" },
  { id: "activeJobs", name: "Running" },
  { id: "erroredJobs", name: "Failed" },
] as const;

function IndexRoute() {
  const {
    data: runs,
    isLoading,
    isError,
    status,
  } = useLiveQuery({
    query: (q) => q.from({ workflowRun: workflowRuns }).orderBy(({ workflowRun }) => workflowRun.created_at, "desc"),
  });
  const hasActiveWork = runs.some(
    (workflowRun) => workflowRun.status === "running" || workflowRun.active_job_count > 0,
  );
  const now = useLiveClock(hasActiveWork);
  const [theme, toggleTheme] = useTheme();

  return (
    <main className="mx-auto flex min-h-[calc(100vh-2rem)] w-full max-w-7xl flex-col gap-6 px-6 py-10">
      <header className="flex items-start justify-between gap-4">
        <div>
          <p className="mb-2 text-sm font-medium uppercase tracking-wide text-app-foreground-muted">Runs</p>
          <h1 className="text-3xl font-semibold tracking-tight text-app-foreground">Workflow runs</h1>
          <p className="mt-2 text-app-foreground-muted">
            Status, job counts, and basic metadata for each workflow run.
          </p>
        </div>
        <Button
          variant="outline"
          className="shrink-0"
          type="button"
          onClick={toggleTheme}
          aria-label={`Switch to ${theme === "light" ? "dark" : "light"} theme`}
        >
          {theme === "light" ? "Dark mode" : "Light mode"}
        </Button>
      </header>

      {isLoading && <p className="text-app-foreground-muted">Loading workflow runs…</p>}
      {isError && (
        <p className="rounded-md border border-app-danger/30 bg-app-danger/10 p-4 text-app-danger" role="alert">
          Unable to load workflow runs (status: {status}).
        </p>
      )}

      {runs.length > 0 ? (
        <Table aria-label="Workflow runs" className="max-h-[calc(100vh-18rem)] min-h-0 flex-1">
          <TableHeader columns={columns}>
            {(column) => (
              <Column id={column.id} isRowHeader={column.id === "workflowRunId"}>
                {column.name}
              </Column>
            )}
          </TableHeader>
          <TableBody items={runs}>
            {(workflowRun) => <WorkflowRunRow workflowRun={workflowRun} now={now} />}
          </TableBody>
        </Table>
      ) : (
        !isLoading &&
        !isError && (
          <p className="rounded-md border border-dashed border-app-border p-8 text-center text-app-foreground-muted">
            No workflow runs loaded.
          </p>
        )
      )}
    </main>
  );
}

function WorkflowRunRow({ workflowRun, now }: { workflowRun: WorkflowRunSummary; now: number }) {
  const totalJobs =
    workflowRun.active_job_count + workflowRun.succeeded_job_count + workflowRun.errored_job_count;

  return (
    <Row
      id={workflowRun.workflow_run_id}
      textValue={`${workflowRun.workflow_run_id} ${statusLabel(workflowRun.status)}`}
    >
      <Cell className="min-w-56 font-mono text-xs" textValue={workflowRun.workflow_run_id}>
        {workflowRun.workflow_run_id}
      </Cell>
      <Cell textValue={statusLabel(workflowRun.status)}>
        <StatusBadge status={workflowRun.status} />
      </Cell>
      <Cell textValue={formatDate(workflowRun.created_at)}>{formatDate(workflowRun.created_at)}</Cell>
      <Cell textValue={formatDate(workflowRun.started_at)}>{formatDate(workflowRun.started_at)}</Cell>
      <Cell textValue={formatDate(workflowRun.completed_at)}>{formatDate(workflowRun.completed_at)}</Cell>
      <Cell textValue={formatWorkflowDuration(workflowRun.started_at, workflowRun.completed_at, now)}>
        {formatWorkflowDuration(workflowRun.started_at, workflowRun.completed_at, now)}
      </Cell>
      <Cell textValue={String(totalJobs)}>{totalJobs}</Cell>
      <Cell textValue={String(workflowRun.succeeded_job_count)}>{workflowRun.succeeded_job_count}</Cell>
      <Cell textValue={String(workflowRun.active_job_count)}>{workflowRun.active_job_count}</Cell>
      <Cell textValue={String(workflowRun.errored_job_count)}>{workflowRun.errored_job_count}</Cell>
    </Row>
  );
}

function StatusBadge({ status }: { status: string }) {
  return (
    <span className={`inline-flex rounded-full px-2.5 py-1 text-xs font-medium ${statusClasses(status)}`}>
      {statusLabel(status)}
    </span>
  );
}

function statusLabel(status: string): string {
  return status.replace(/_/g, " ");
}

function statusClasses(status: string): string {
  switch (status) {
    case "succeeded":
      return "bg-app-success/15 text-app-success";
    case "running":
      return "bg-app-warning/15 text-app-warning";
    case "failed":
    case "errored":
      return "bg-app-danger/15 text-app-danger";
    default:
      return "bg-app-border/50 text-app-foreground-muted";
  }
}

function formatDate(value: string | number | null): string {
  if (value === null) {
    return "—";
  }

  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? String(value)
    : new Intl.DateTimeFormat(undefined, {
        dateStyle: "medium",
        timeStyle: "short",
      }).format(date);
}

function useLiveClock(enabled: boolean): number {
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    if (!enabled) {
      return;
    }

    setNow(Date.now());
    const interval = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(interval);
  }, [enabled]);

  return now;
}

function formatWorkflowDuration(
  startedAt: number | null,
  completedAt: number | null,
  now: number,
): string {
  if (startedAt === null) {
    return "—";
  }

  return formatDuration(Math.max(0, (completedAt ?? now) - startedAt));
}

function formatDuration(durationMs: number | null): string {
  if (durationMs === null) {
    return "—";
  }

  const totalSeconds = Math.floor(durationMs / 1000);
  if (totalSeconds < 60) {
    return `${totalSeconds}s`;
  }

  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  if (minutes < 60) {
    return `${minutes}m ${seconds}s`;
  }

  const hours = Math.floor(minutes / 60);
  return `${hours}h ${minutes % 60}m`;
}
