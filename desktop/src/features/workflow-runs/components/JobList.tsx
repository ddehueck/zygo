import { JobRun } from "@/bindings";
import { formatDuration } from "@/lib/dates";
import { useNavigate } from "@tanstack/react-router";
import { StatusIcon, statusLabel } from "./statuses";

export function JobList({ jobs }: { jobs: JobRun[] }) {
  const navigate = useNavigate();

  return (
    <div className="mt-2 overflow-x-auto">
      <div className="grid min-w-136 grid-cols-[3rem_minmax(12rem,1fr)_9rem_7rem] items-center border-b border-app-border px-3 py-2 text-xs font-medium text-app-foreground-muted">
        <span>#</span>
        <span>Job</span>
        <span>Status</span>
        <span>Duration</span>
      </div>
      <div role="list" aria-label="Jobs">
        {jobs.map((job, index) => (
          <button
            key={job.id}
            type="button"
            role="listitem"
            className="group grid w-full min-w-136 grid-cols-[3rem_minmax(12rem,1fr)_9rem_7rem] items-center border-b border-app-border px-3 py-3 text-left transition hover:bg-app-interaction-hover focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-app-accent"
            onClick={() =>
              void navigate({
                to: "/runs/$workflowRunId/jobs/$jobId",
                params: { workflowRunId: job.workflow_run_id, jobId: job.id },
              })
            }
          >
            <span className="text-sm text-app-foreground-muted">{index + 1}</span>
            <span className="truncate pr-4 font-mono text-sm font-medium text-app-foreground">
              {job.job_id}
            </span>
            <span>
              <JobStatus status={job.status} />
            </span>
            <span className="font-mono text-xs text-app-foreground-muted">
              {job.duration_ms === null ? "—" : formatDuration(job.duration_ms)}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}

function JobStatus({ status }: { status: string }) {
  return (
    <span className="inline-flex items-center gap-1.5 text-xs text-app-foreground-muted">
      <StatusIcon status={status} />
      {statusLabel(status)}
    </span>
  );
}
