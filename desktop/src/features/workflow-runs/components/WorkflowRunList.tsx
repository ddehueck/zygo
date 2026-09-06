import { useLocation, useNavigate } from "@tanstack/react-router";
import { GridList, GridListItem } from "@/components/GridList";
import { useDuration } from "@/hooks/use-duration";
import { shortRunId } from "@/features/workflow-runs/lib/id";
import { JobCountsBadge } from "@/features/workflow-runs/components/JobCountsBadge";
import { TagOverflowList } from "@/features/workflow-runs/components/TagOverflowList";
import { StatusIcon as StatusGlyph } from "./statuses";
import type { WorkflowRunListData } from "@/features/workflow-runs/hooks/use-workflow-runs-list-data";
import { useWorkflowRunListHotkeys } from "@/features/workflow-runs/hooks/use-workflow-run-list-hotkeys";
import { useWorkflowRunSearch } from "@/features/workflow-runs/search/WorkflowRunSearchContext";

type WorkflowRunListProps = { runs: WorkflowRunListData };
type WorkflowRunListRow = WorkflowRunListData[number];

export function WorkflowRunList({ runs }: WorkflowRunListProps) {
  const navigate = useNavigate();
  const location = useLocation();
  const listRef = useWorkflowRunListHotkeys();
  const { applyFilters } = useWorkflowRunSearch();

  const filteredRuns = applyFilters(runs);

  return (
    <GridList
      ref={listRef}
      aria-label="Workflow runs"
      items={filteredRuns}
      onAction={(key) =>
        navigate({
          to: "/runs/$workflowRunId",
          params: { workflowRunId: String(key) },
          // Browsers do not expose the previous history entry, so record the exact
          // list location on the detail entry before navigating away.
          state: (previous) => ({
            ...previous,
            breadcrumbBack: {
              href: location.href,
              pathname: location.pathname,
              historyIndex: location.state.__TSR_index,
            },
          }),
        })
      }
    >
      {(workflowRun) => (
        <WorkflowRunItem id={workflowRun.workflowRun.public_id} item={workflowRun} />
      )}
    </GridList>
  );
}

function WorkflowRunItem({ id, item }: { id: string; item: WorkflowRunListRow }) {
  const { workflowRun, tags } = item;

  const duration = useDuration({
    startedAt: workflowRun.started_at,
    completedAt: workflowRun.completed_at,
  });

  const textValue = `${workflowRun.public_id} ${statusLabel(workflowRun.status)}`;

  return (
    <GridListItem id={id} textValue={textValue}>
      <div className="grid min-w-0 grid-cols-[minmax(0,1fr)_minmax(0,2fr)_6rem_max-content] items-center gap-4 px-1.5">
        <div className="flex min-w-0 items-center gap-3 py-1">
          <StatusIcon status={workflowRun.status} />
          <p className="min-w-0 truncate font-mono text-base font-semibold tracking-tight text-app-foreground">
            <span className="text-app -foreground-muted font-normal">
              ({shortRunId(workflowRun.public_id)})
            </span>{" "}
            {workflowRun.workflow_id}
          </p>
        </div>

        <div className="flex min-w-0 justify-end">
          <TagOverflowList tags={tags} />
        </div>

        <div className="flex min-w-0 justify-end">
          <p className="w-full text-right font-mono text-sm text-app-foreground-muted">
            {duration ?? "—"}
          </p>
        </div>

        <div className="flex min-w-0 justify-end">
          <JobCountsBadge
            activeJobCount={workflowRun.active_job_count}
            succeededJobCount={workflowRun.succeeded_job_count}
            erroredJobCount={workflowRun.errored_job_count}
          />
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

function statusLabel(status: string): string {
  return status.replace(/_/g, " ");
}
