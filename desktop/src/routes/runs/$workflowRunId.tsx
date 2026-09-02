import { useLiveQuery } from "@tanstack/react-db";
import { useEffect, useState, type ReactNode } from "react";
import { createFileRoute, Link } from "@tanstack/react-router";
import type { WorkflowRunSummary } from "../../bindings";
import { workflowRuns } from "../../db/workflow-run-summaries";

export const Route = createFileRoute("/runs/$workflowRunId")({
  beforeLoad: ({ params }) => ({
    breadcrumb: {
      label: "Run details",
      link: `/runs/${params.workflowRunId}`,
    },
  }),
  component: RunRoute,
});

function RunRoute() {
  const { workflowRunId } = Route.useParams();
  const { data: runs, isLoading, isError, status } = useLiveQuery({
    query: (q) => q.from({ workflowRun: workflowRuns }),
  });
  const workflowRun = runs.find((run) => run.workflow_run_id === workflowRunId);
  const isActive =
    workflowRun !== undefined && (workflowRun.status === "running" || workflowRun.active_job_count > 0);
  const now = useLiveClock(isActive);

  if (isLoading) {
    return <RunPageShell><p className="text-app-foreground-muted">Loading workflow run…</p></RunPageShell>;
  }

  if (isError) {
    return (
      <RunPageShell>
        <p className="rounded-md border border-app-danger/30 bg-app-danger/10 p-4 text-app-danger" role="alert">
          Unable to load workflow run (status: {status}).
        </p>
      </RunPageShell>
    );
  }

  if (!workflowRun) {
    return (
      <RunPageShell>
        <div className="rounded-lg border border-dashed border-app-border p-8 text-center">
          <h1 className="text-lg font-semibold text-app-foreground">Workflow run not found</h1>
          <p className="mt-2 text-app-foreground-muted">The requested run may have been removed.</p>
          <Link to="/" className="mt-5 inline-block text-sm font-medium text-app-accent hover:underline">
            Back to workflow runs
          </Link>
        </div>
      </RunPageShell>
    );
  }

  return <RunDetails workflowRun={workflowRun} isActive={isActive} now={now} />;
}

function RunPageShell({ children }: { children: ReactNode }) {
  return (
    <main className="mx-auto flex w-full max-w-4xl flex-col gap-6 px-6">
      <Link to="/" className="w-fit text-sm font-medium text-app-foreground-muted hover:text-app-foreground">
        ← Back to workflow runs
      </Link>
      {children}
    </main>
  );
}

function RunDetails({
  workflowRun,
  isActive,
  now,
}: {
  workflowRun: WorkflowRunSummary;
  isActive: boolean;
  now: number;
}) {
  const totalJobs =
    workflowRun.active_job_count + workflowRun.succeeded_job_count + workflowRun.errored_job_count;
  const duration = formatWorkflowDuration(workflowRun.started_at, workflowRun.completed_at, now);

  return (
    <main className="mx-auto flex w-full max-w-4xl flex-col gap-6 px-6 py-10">
      <Link to="/" className="w-fit text-sm font-medium text-app-foreground-muted hover:text-app-foreground">
        ← Back to workflow runs
      </Link>

      <header>
        <p className="mb-2 text-sm font-medium uppercase tracking-wide text-app-foreground-muted">Run details</p>
        <div className="flex flex-wrap items-center gap-3">
          <h1 className="font-mono text-2xl font-semibold tracking-tight text-app-foreground">{workflowRun.workflow_run_id}</h1>
          <StatusBadge status={workflowRun.status} />
        </div>
        <p className="mt-2 text-app-foreground-muted">General information and job counts for this workflow run.</p>
      </header>

      <section className="rounded-lg border border-app-border bg-app-bg-elevated p-5" aria-labelledby="run-information-heading">
        <h2 id="run-information-heading" className="text-lg font-semibold text-app-foreground">General information</h2>
        <dl className="mt-5 grid gap-5 sm:grid-cols-2">
          <InfoItem label="Workflow run ID" value={workflowRun.workflow_run_id} mono />
          <InfoItem label="Status" value={statusLabel(workflowRun.status)} />
          <InfoItem label="Created" value={formatDate(workflowRun.created_at)} />
          <InfoItem label="Started" value={formatDate(workflowRun.started_at)} />
          <InfoItem label="Completed" value={formatDate(workflowRun.completed_at)} />
          <InfoItem label="Duration" value={duration} />
          <InfoItem label="Last updated" value={formatDate(workflowRun.updated_at)} />
        </dl>
      </section>

      <section aria-labelledby="job-summary-heading">
        <h2 id="job-summary-heading" className="text-lg font-semibold text-app-foreground">Job summary</h2>
        <div className="mt-3 grid gap-3 sm:grid-cols-4">
          <SummaryItem label="Total jobs" value={totalJobs} />
          <SummaryItem label="Succeeded" value={workflowRun.succeeded_job_count} />
          <SummaryItem label="Running" value={workflowRun.active_job_count} />
          <SummaryItem label="Failed" value={workflowRun.errored_job_count} />
        </div>
        {isActive && <p className="mt-4 text-sm text-app-foreground-muted">This run is still in progress.</p>}
      </section>
    </main>
  );
}

function InfoItem({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) {
  return (
    <div>
      <dt className="text-sm text-app-foreground-muted">{label}</dt>
      <dd className={`mt-1 break-all text-sm text-app-foreground ${mono ? "font-mono text-xs" : ""}`}>{value}</dd>
    </div>
  );
}

function SummaryItem({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-lg border border-app-border bg-app-bg-elevated p-4">
      <p className="text-sm text-app-foreground-muted">{label}</p>
      <p className="mt-2 text-2xl font-semibold text-app-foreground">{value}</p>
    </div>
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

function formatWorkflowDuration(startedAt: number | null, completedAt: number | null, now: number): string {
  if (startedAt === null) {
    return "—";
  }

  return formatDuration(Math.max(0, (completedAt ?? now) - startedAt));
}

function useLiveClock(enabled: boolean): number {
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    if (!enabled) {
      return;
    }

    const interval = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(interval);
  }, [enabled]);

  return now;
}

function formatDuration(durationMs: number): string {
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
