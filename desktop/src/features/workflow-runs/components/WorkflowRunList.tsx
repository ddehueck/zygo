import { useHotkey } from "@tanstack/react-hotkeys";
import { useNavigate } from "@tanstack/react-router";

import { useMemo, useRef, useState } from "react";
import { GridList, GridListItem } from "@/components/GridList";
import type { WorkflowSearchToken } from "@/features/workflow-runs/components/search/search-token-field-value";
import { useDuration } from "@/hooks/use-duration";
import { shortRunId } from "@/features/workflow-runs/lib/id";
import { JobCountsBadge } from "@/features/workflow-runs/components/JobCountsBadge";
import { TagOverflowList } from "@/features/tags/components/TagOverflowList";
import { Icons } from "@/components/icons";
import type { useWorkflowRunsListData } from "@/features/workflow-runs/hooks/use-workflow-runs-list-data";
import { TokenSearch } from "@/features/token-search/TokenSearch";

type WorkflowRunListData = ReturnType<typeof useWorkflowRunsListData>["data"];

type WorkflowRunListProps = { runs: WorkflowRunListData };
type WorkflowRunListRow = WorkflowRunListData[number];

export function WorkflowRunList({ runs }: WorkflowRunListProps) {
  const navigate = useNavigate();
  const listRef = useRef<HTMLDivElement>(null);
  const [searchTokens, setSearchTokens] = useState<WorkflowSearchToken[]>([]);
  const filteredRuns = useMemo(
    () => runs.filter((run) => matchesSearch(run, searchTokens)),
    [runs, searchTokens],
  );

  const focusList = () => {
    const list = listRef.current;
    if (list && !list.contains(document.activeElement)) {
      list.focus();
    }
  };

  useHotkey("ArrowUp", focusList);
  useHotkey("ArrowDown", focusList);

  return (
    <div className="w-full">
      <div className="border-b border-app-border px-2 py-2">
        <TokenSearch onSearchChange={setSearchTokens} />
      </div>
      <GridList
        ref={listRef}
        aria-label="Workflow runs"
        items={filteredRuns}
        onAction={(key) =>
          navigate({
            to: "/runs/$workflowRunId",
            params: { workflowRunId: String(key) },
          })
        }
      >
        {(workflowRun) => <WorkflowRunItem id={workflowRun.workflowRun.id} item={workflowRun} />}
      </GridList>
      {filteredRuns.length === 0 && searchTokens.length > 0 && (
        <p className="px-4 py-8 text-center text-sm text-app-foreground-muted">
          No workflow runs match these filters.
        </p>
      )}
    </div>
  );
}

function matchesSearch(run: WorkflowRunListRow, searchTokens: WorkflowSearchToken[]): boolean {
  return searchTokens.every((token) => {
    switch (token.type) {
      case "workflow":
        return run.workflowRun.workflow_id.toLowerCase() === token.workflowId.toLowerCase();
      case "tag":
        return run.tags.some(
          (tag) =>
            tag.key.toLowerCase() === token.name.toLowerCase() &&
            (token.value === undefined || tag.value.toLowerCase() === token.value.toLowerCase()),
        );
    }
  });
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
      <div className="grid min-w-0 grid-cols-[minmax(0,1fr)_minmax(0,2fr)_6rem_max-content] items-center gap-4 px-1.5">
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
