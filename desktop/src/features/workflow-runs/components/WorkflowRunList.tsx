import { useHotkey } from "@tanstack/react-hotkeys";
import { useNavigate } from "@tanstack/react-router";

import { useRef } from "react";
import { GridList, GridListItem } from "../../../components/GridList";
import { sum } from "../../../lib/math";
import { useDuration } from "../../../hooks/use-duration";
import { shortRunId } from "../lib/id";
import { JobCountsBadge } from "./JobCountsBadge";
import { TagOverflowList } from "../../tags/components/TagOverflowList";
import { Icons } from "../../../components/icons";
import type { useWorkflowRunsListData } from "../hooks/use-workflow-runs-list-data";

type WorkflowRunListData = ReturnType<typeof useWorkflowRunsListData>["data"];

type WorkflowRunListProps = { runs: WorkflowRunListData };
type WorkflowRunListRow = WorkflowRunListData[number];

export function WorkflowRunList({ runs }: WorkflowRunListProps) {
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
      {(workflowRun) => <WorkflowRunItem id={workflowRun.workflowRun.id} item={workflowRun} />}
    </GridList>
  );
}

function WorkflowRunItem({ id, item }: { id: string; item: WorkflowRunListRow }) {
  const { workflowRun, tags } = item;

  const duration = useDuration({
    startedAt: workflowRun.started_at,
    completedAt: workflowRun.completed_at,
  });

  const textValue = `${workflowRun.id} ${statusLabel(workflowRun.status)}`;

  return (
    <GridListItem id={id} textValue={textValue}>
      <div className="grid min-w-0 grid-cols-[minmax(0,1fr)_minmax(0,2fr)_8rem_4rem] items-center gap-6 px-1.5">
        <div className="flex min-w-0 items-center gap-3 py-1">
          <StatusIcon status={workflowRun.status} />
          <p className="min-w-0 truncate font-mono text-base font-semibold tracking-tight text-app-foreground">
            <span className="text-app -foreground-muted font-normal">
              ({shortRunId(workflowRun.id)})
            </span>{" "}
            {workflowRun.workflow_id}
          </p>
        </div>

        <div className="flex min-w-0 justify-end">
          <TagOverflowList tags={tags} />
        </div>

        <div className="flex justify-end">
          <div className="flex flex-1 items-center justify-start">
            <JobCountsBadge
              activeJobCount={workflowRun.active_job_count}
              succeededJobCount={workflowRun.succeeded_job_count}
              erroredJobCount={workflowRun.errored_job_count}
            />
          </div>
        </div>

        <div className="flex justify-end py-1">
          <p className="font-mono text-xs text-app-foreground">{duration ?? "—"}</p>
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
