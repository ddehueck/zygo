import { useHotkey } from "@tanstack/react-hotkeys";
import { useNavigate } from "@tanstack/react-router";
import { useRef } from "react";
import { WorkflowRun } from "../../../bindings";
import { GridList, GridListItem } from "../../../components/GridList";
import { sum } from "../../../lib/math";
import { useDuration } from "../../../hooks/use-duration";
import { shortRunId } from "../lib/id";
import { JobCountsBadge } from "./JobCountsBadge";
import { Icons } from "../../../components/icons";

export function WorkflowRunList({ runs }: { runs: WorkflowRun[] }) {
  const navigate = useNavigate();
  const listRef = useRef<HTMLDivElement>(null);

  const focusList = () => {
    const list = listRef.current;
    if (list && !list.contains(document.activeElement)) {
      list.focus();
    }
  };

  useHotkey("ArrowUp", focusList);
  useHotkey("ArrowDown", focusList);

  return (
    <GridList
      ref={listRef}
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
