import { eq } from "@tanstack/db";
import { useLiveQuery } from "@tanstack/react-db";
import { useEffect, useState } from "react";
import { createFileRoute } from "@tanstack/react-router";
import type { JobRunSummary, WorkflowRunSummary } from "../bindings";
import { jobRuns } from "../db/job-run-summaries";
import { workflowRuns } from "../db/workflow-run-summaries";

export const Route = createFileRoute("/")({
  component: IndexRoute,
});

type JoinedRun = {
  workflowRun: WorkflowRunSummary;
  jobRun: JobRunSummary | undefined;
};

type WorkflowRunWithJobs = {
  workflowRun: WorkflowRunSummary;
  jobRuns: JobRunSummary[];
};

function IndexRoute() {
  const {
    data: joinedRuns,
    isLoading,
    isError,
    status,
  } = useLiveQuery({
    query: (q) =>
      q
        .from({ workflowRun: workflowRuns })
        .leftJoin({ jobRun: jobRuns }, ({ workflowRun, jobRun }) =>
          eq(workflowRun.workflow_run_id, jobRun.workflow_run_id),
        )
        .orderBy(({ workflowRun }) => workflowRun.created_at, "desc"),
  });

  const runs = groupRuns(joinedRuns);
  const hasActiveWork = runs.some(
    ({ workflowRun, jobRuns }) =>
      workflowRun.status === "running" ||
      workflowRun.active_job_count > 0 ||
      jobRuns.some((jobRun) => jobRun.status === "running"),
  );
  const now = useLiveClock(hasActiveWork);

  return (
    <main className="mx-auto flex min-h-screen w-full max-w-5xl flex-col gap-6 px-6 py-10">
      <header>
        <p className="mb-2 text-sm font-medium uppercase tracking-wide text-slate-500">Runs</p>
        <h1 className="text-3xl font-semibold tracking-tight text-slate-950">Workflow runs</h1>
        <p className="mt-2 text-slate-600">
          A quick summary of each workflow and the jobs it contains.
        </p>
      </header>

      {isLoading && <p className="text-slate-600">Loading workflow runs…</p>}
      {isError && (
        <p className="rounded-md border border-red-200 bg-red-50 p-4 text-red-700" role="alert">
          Unable to load workflow runs (status: {status}).
        </p>
      )}

      {runs.length > 0 ? (
        <ul className="flex flex-col gap-4">
          {runs.map(({ workflowRun, jobRuns }) => (
            <li key={workflowRun.workflow_run_id}>
              <WorkflowRunCard workflowRun={workflowRun} jobRuns={jobRuns} now={now} />
            </li>
          ))}
        </ul>
      ) : (
        !isLoading &&
        !isError && (
          <p className="rounded-md border border-dashed border-slate-300 p-8 text-center text-slate-600">
            No workflow runs loaded.
          </p>
        )
      )}
    </main>
  );
}

function groupRuns(joinedRuns: JoinedRun[]): WorkflowRunWithJobs[] {
  const groupedRuns = new Map<string, WorkflowRunWithJobs>();

  for (const { workflowRun, jobRun } of joinedRuns) {
    let groupedRun = groupedRuns.get(workflowRun.workflow_run_id);

    if (!groupedRun) {
      groupedRun = { workflowRun, jobRuns: [] };
      groupedRuns.set(workflowRun.workflow_run_id, groupedRun);
    }

    if (jobRun) {
      groupedRun.jobRuns.push(jobRun);
    }
  }

  return Array.from(groupedRuns.values());
}

function WorkflowRunCard({ workflowRun, jobRuns, now }: WorkflowRunWithJobs & { now: number }) {
  const totalJobs =
    workflowRun.active_job_count + workflowRun.succeeded_job_count + workflowRun.errored_job_count;

  return (
    <article className="overflow-hidden rounded-lg border border-slate-200 bg-white shadow-sm">
      <div className="flex flex-wrap items-start justify-between gap-4 border-b border-slate-200 px-5 py-4">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-3">
            <h2 className="font-mono text-sm font-semibold text-slate-900">
              {workflowRun.workflow_run_id}
            </h2>
            <StatusBadge status={workflowRun.status} />
          </div>
          <p className="mt-2 text-sm text-slate-500">
            Created {formatDate(workflowRun.created_at)}
          </p>
        </div>
        <div className="text-left text-sm sm:text-right">
          <p className="text-xs font-medium uppercase tracking-wide text-slate-500">Duration</p>
          <p className="mt-1 font-medium text-slate-900">
            {formatWorkflowDuration(workflowRun.started_at, workflowRun.completed_at, now)}
          </p>
        </div>
      </div>

      <dl className="grid grid-cols-2 gap-4 border-b border-slate-200 px-5 py-4 sm:grid-cols-4">
        <SummaryStat label="Total jobs" value={totalJobs} />
        <SummaryStat label="Succeeded" value={workflowRun.succeeded_job_count} />
        <SummaryStat label="Running" value={workflowRun.active_job_count} />
        <SummaryStat label="Failed" value={workflowRun.errored_job_count} />
      </dl>

      <div className="px-5 py-4">
        <h3 className="text-sm font-semibold text-slate-900">Jobs ({jobRuns.length})</h3>
        {jobRuns.length > 0 ? (
          <div className="mt-3 overflow-x-auto">
            <table className="w-full min-w-120 text-left text-sm">
              <thead className="border-b border-slate-200 text-xs uppercase tracking-wide text-slate-500">
                <tr>
                  <th className="px-2 py-2 font-medium">Job</th>
                  <th className="px-2 py-2 font-medium">Status</th>
                  <th className="px-2 py-2 font-medium">Duration</th>
                  <th className="px-2 py-2 text-right font-medium">Retries</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {jobRuns.map((jobRun) => (
                  <tr key={jobRun.id}>
                    <td className="px-2 py-3 font-mono text-xs text-slate-700">{jobRun.job_id}</td>
                    <td className="px-2 py-3">
                      <StatusBadge status={jobRun.status} />
                    </td>
                    <td className="px-2 py-3 text-slate-600">{formatJobDuration(jobRun, now)}</td>
                    <td className="px-2 py-3 text-right text-slate-600">{jobRun.retry_count}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="mt-3 text-sm text-slate-500">
            {totalJobs > 0
              ? "No job run details are currently loaded."
              : "No jobs recorded for this workflow run."}
          </p>
        )}
      </div>
    </article>
  );
}

function SummaryStat({ label, value }: { label: string; value: number }) {
  return (
    <div>
      <dt className="text-xs font-medium uppercase tracking-wide text-slate-500">{label}</dt>
      <dd className="mt-1 text-xl font-semibold text-slate-950">{value}</dd>
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  return (
    <span
      className={`inline-flex rounded-full px-2.5 py-1 text-xs font-medium ${statusClasses(status)}`}
    >
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
      return "bg-emerald-100 text-emerald-700";
    case "running":
      return "bg-amber-100 text-amber-700";
    case "failed":
    case "errored":
      return "bg-red-100 text-red-700";
    default:
      return "bg-slate-100 text-slate-700";
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

function formatJobDuration(jobRun: JobRunSummary, now: number): string {
  if (jobRun.status !== "running") {
    return formatDuration(jobRun.duration_ms);
  }

  const startedAt = parseTimestamp(jobRun.updated_at) ?? parseTimestamp(jobRun.created_at);
  return startedAt === null ? "—" : formatDuration(Math.max(0, now - startedAt));
}

function parseTimestamp(value: string): number | null {
  const timestamp = new Date(value).getTime();
  return Number.isNaN(timestamp) ? null : timestamp;
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
