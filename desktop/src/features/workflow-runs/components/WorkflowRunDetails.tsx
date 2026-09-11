import { useLiveQuery } from "@tanstack/react-db";
import { eq } from "@tanstack/db";
import { Link, useNavigate } from "@tanstack/react-router";
import { type ReactNode } from "react";

import type { JobRun, Tag, Workflow, WorkflowRun } from "@/bindings";
import {
  dataReferencesCollection,
  jobRunsCollection,
  logsCollection,
  tagsCollection,
  workflowRunsCollection,
  workflowsCollection,
} from "@/db/collections";

import { Button } from "@/components/Button";
import { Icon, iconDefinitions } from "@/components/icons";
import { useDuration } from "@/hooks/use-duration";
import { getFileNameFromUri } from "@/features/data-viewer/lib/file-name";
import { useWatchLogs } from "@/features/log-viewer/hooks/use-watch-logs";
import { formatDate } from "@/lib/dates";
import { RunStatus, StatusIcon, isTerminalWorkflowRunStatus, statusLabel } from "./statuses";
import { TagBadge } from "./TagBadge";
import { Heading, Text } from "@/components/Text";
import { sum } from "@/lib/math";

type WorkflowRunDetailsProps = {
  workflowRunId: string;
};

export function WorkflowRunDetails({ workflowRunId }: WorkflowRunDetailsProps) {
  const runsQuery = useLiveQuery({
    query: (q) =>
      q
        .from({ workflowRun: workflowRunsCollection })
        .leftJoin(
          { workflow: workflowsCollection },
          ({ workflowRun, workflow }) => eq(workflowRun.workflow_id, workflow.id),
        )
        .where(({ workflowRun }) => eq(workflowRun.id, Number(workflowRunId)))
        .findOne(),
  });
  const jobsQuery = useLiveQuery({
    query: (q) => q.from({ jobRun: jobRunsCollection }),
  });
  const tagsQuery = useLiveQuery({
    query: (q) => q.from({ tag: tagsCollection }),
  });

  const workflowRun = runsQuery.data?.workflowRun;
  const workflow = runsQuery.data?.workflow;
  const jobs = jobsQuery.data
    .filter((job) => job.workflow_run_id === workflowRun?.id)
    .sort((a, b) => a.created_at.localeCompare(b.created_at));
  const runTags = tagsQuery.data.filter((tag) => tag.workflow_run_id === workflowRun?.id);

  if (runsQuery.isLoading) {
    return (
      <RunPageShell>
        <p className="text-app-foreground-muted">Loading workflow run…</p>
      </RunPageShell>
    );
  }

  if (runsQuery.isError) {
    return (
      <RunPageShell>
        <p className="text-app-danger" role="alert">
          Unable to load workflow run (status: {runsQuery.status}).
        </p>
      </RunPageShell>
    );
  }

  if (!workflowRun) {
    return (
      <RunPageShell>
        <h1 className="text-xl font-semibold text-app-foreground">Workflow run not found</h1>
        <p className="mt-2 text-app-foreground-muted">The requested run may have been removed.</p>
        <Link
          to="/"
          className="mt-5 inline-block text-sm font-medium text-app-accent hover:underline"
        >
          Back to workflow runs
        </Link>
      </RunPageShell>
    );
  }

  return (
    <WorkflowRunOverview run={workflowRun} workflow={workflow} jobs={jobs} runTags={runTags} />
  );
}

function RunPageShell({ children }: { children: ReactNode }) {
  return <main className="mx-auto w-full max-w-5xl px-6 py-10">{children}</main>;
}

function WorkflowRunOverview({
  run,
  workflow,
  jobs,
  runTags,
}: {
  run: WorkflowRun;
  workflow: Workflow | undefined;
  jobs: JobRun[];
  runTags: Tag[];
}) {
  const navigate = useNavigate();
  const totalJobs = sum([run.active_job_count, run.succeeded_job_count, run.errored_job_count]);

  const duration = useDuration({
    startedAt: run.started_at,
    completedAt: run.completed_at,
  });

  return (
    <main className="mx-auto w-full max-w-5xl px-6 py-10">
      <header className="">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div className="flex w-full min-w-0 flex-wrap items-center justify-between gap-3">
            <Heading text={workflow?.name ?? ""} />
            <div className="flex items-center gap-3">
              <RunStatus status={run.status} />
              <Button
                variant="secondary"
                onPress={() =>
                  void navigate({
                    to: "/workflows/$workflowId/runs/new",
                    params: { workflowId: String(run.workflow_id) },
                  })
                }
              >
                Run workflow
              </Button>
            </div>
          </div>
        </div>
        <p className="mt-4 flex flex-wrap items-center gap-x-2 gap-y-1 text-sm text-app-foreground-muted">
          <span>{formatDate(run.started_at)}</span>
          <span aria-hidden>•</span>
          <span>{duration ?? "—"}</span>
          <span aria-hidden>•</span>
          <span>{totalJobs} jobs</span>
        </p>
        {runTags.length > 0 && (
          <div className="mt-4 flex flex-wrap gap-x-3 gap-y-1 text-sm text-app-foreground-muted">
            {runTags.map((tag) => (
              <TagBadge key={tag.id} value={tag.value} />
            ))}
          </div>
        )}
      </header>

      <section aria-label="Workflow run previews" className="mt-8 grid gap-4 lg:grid-cols-3">
        <JobsPreviewCard jobs={jobs} run={run} totalJobs={totalJobs} />
        <DataPreviewCard run={run} />
        <LogsPreviewCard run={run} />
      </section>
    </main>
  );
}

function PreviewCard({
  title,
  summary,
  children,
  footer,
}: {
  title: string;
  summary: string;
  children: ReactNode;
  footer: string;
}) {
  return (
    <section className="flex h-full min-h-80 min-w-0 flex-col overflow-hidden rounded-xl border border-app-border bg-app-bg-surface p-5 group-hover:border-app-accent group-hover:shadow-md motion-safe:transition-[box-shadow,border-color] motion-safe:duration-150 motion-safe:ease-out">
      <header className="pb-4">
        <Heading text={title} size="medium" />
        <p className="mt-1 text-sm text-app-foreground-muted">{summary}</p>
      </header>
      <div className="flex-1">{children}</div>
      <footer className="pt-4 text-sm font-medium text-app-foreground-secondary">
        {footer} <span aria-hidden>→</span>
      </footer>
    </section>
  );
}

function PreviewRow({ label, value }: { label: ReactNode; value: ReactNode }) {
  return (
    <div className="grid min-w-0 grid-cols-[auto_minmax(0,1fr)] items-start gap-3 py-3 first:pt-4 last:pb-4">
      <span className="min-w-0 text-sm whitespace-nowrap text-app-foreground-muted">{label}</span>
      <span className="min-w-0 overflow-hidden text-right text-sm font-medium text-app-foreground">
        {value}
      </span>
    </div>
  );
}

function JobsPreviewCard({
  jobs,
  run,
  totalJobs,
}: {
  jobs: JobRun[];
  run: WorkflowRun;
  totalJobs: number;
}) {
  const visibleJobs = jobs.slice(0, 4);
  const summary =
    run.errored_job_count > 0
      ? `${run.errored_job_count} job${run.errored_job_count === 1 ? "" : "s"} failed`
      : `${run.succeeded_job_count} of ${totalJobs} jobs completed`;

  return (
    <Link
      to="/runs/$workflowRunId/jobs"
      params={{ workflowRunId: String(run.id) }}
      aria-label="View all jobs for this workflow run"
      className="group block h-full rounded-xl outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-app-accent"
    >
      <PreviewCard title="Jobs" summary={summary} footer="View all jobs">
        {visibleJobs.length > 0 ? (
          visibleJobs.map((job) => (
            <PreviewRow
              key={job.id}
              label={
                <span className="flex min-w-0 items-center gap-2">
                  <StatusIcon status={job.status} className="size-3.5 shrink-0" />
                  <span className="block truncate" title={job.job_id}>
                    {job.job_id}
                  </span>
                </span>
              }
              value={statusLabel(job.status)}
            />
          ))
        ) : (
          <p className="py-5 text-sm text-app-foreground-muted">No jobs recorded.</p>
        )}
      </PreviewCard>
    </Link>
  );
}

function DataPreviewCard({ run }: { run: WorkflowRun }) {
  const referencesQuery = useLiveQuery({
    query: (q) =>
      q
        .from({ reference: dataReferencesCollection })
        .where(({ reference }) => eq(reference.workflow_run_id, run.id))
        .orderBy(({ reference }) => reference.id, "desc")
        .limit(5),
  });
  const visibleReferences = referencesQuery.data;

  return (
    <Link
      to="/runs/$workflowRunId/data"
      params={{ workflowRunId: String(run.id) }}
      aria-label="Browse data lineage for this workflow run"
      className="group block h-full rounded-xl outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-app-accent"
    >
      <PreviewCard
        title="Data"
        summary={`${referencesQuery.data.length} file${referencesQuery.data.length === 1 ? "" : "s"} · 42.1 GB`}
        footer="Browse lineage"
      >
        <div>
          {visibleReferences.map((reference) => (
            <div
              key={reference.id}
              className="flex min-w-0 items-center gap-3 py-3 first:pt-4 last:pb-4"
            >
              <Icon
                aria-hidden
                className="size-4 shrink-0 text-app-foreground-muted"
                definition={iconDefinitions.file}
              />
              <span className="min-w-0 flex-1 truncate text-sm font-medium text-app-foreground">
                {getFileNameFromUri(reference.uri)}
              </span>
              <span className="shrink-0 text-xs text-app-foreground-muted">48 KB</span>
            </div>
          ))}
        </div>
      </PreviewCard>
    </Link>
  );
}

function LogsPreviewCard({ run }: { run: WorkflowRun }) {
  const watcher = useWatchLogs({
    workflowRunId: run.id,
    enabled: !isTerminalWorkflowRunStatus(run.status),
  });

  const logsQuery = useLiveQuery({
    query: (q) =>
      q
        .from({ log: logsCollection })
        .where(({ log }) => eq(log.workflow_run_id, run.id))
        .orderBy(({ log }) => log.id, "desc")
        .limit(5),
  });

  return (
    <Link
      to="/runs/$workflowRunId/logs"
      params={{ workflowRunId: String(run.id) }}
      aria-label="View logs for this workflow run"
      className="group block h-full rounded-xl outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-app-accent"
    >
      <PreviewCard
        title="Logs"
        summary={run.errored_job_count > 0 ? "Errors found in this run" : "No errors reported"}
        footer="View run logs"
      >
        {logsQuery.data.length > 0 ? (
          logsQuery.data.map((log) => (
            <div key={log.id} className="min-w-0 py-3 first:pt-4 last:pb-4">
              <span className="block truncate text-sm font-medium text-app-foreground">
                {log.content.trimStart()}
              </span>
            </div>
          ))
        ) : (
          <Text
            className="block py-3"
            size="small"
            variant={watcher.isError ? "danger" : "muted"}
            role={watcher.isError ? "alert" : undefined}
          >
            {watcher.isPending
              ? "Loading logs…"
              : watcher.isError
                ? "Unable to load logs."
                : "No logs recorded."}
          </Text>
        )}
      </PreviewCard>
    </Link>
  );
}
