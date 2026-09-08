import type { Log } from "@/bindings";
import { cn } from "@/components/utils";
import { shortRunId } from "@/features/workflow-runs/lib/id";
import { useLogSearchContext } from "../search/LogSearchContext";

export function LogRow({
  log,
  showDate = true,
  showJobId = true,
}: {
  log: Log;
  showDate?: boolean;
  showJobId?: boolean;
}) {
  const { applyJobRunFilter, jobRunPublicId } = useLogSearchContext();
  const shortId = shortRunId(log.job_run_id);
  const isActiveFilter = jobRunPublicId != null && log.job_run_id === jobRunPublicId;

  return (
    <div
      className="log-viewer-grid grid items-start gap-3 border-b border-app-border/25 px-3 py-1 font-mono text-xs leading-[18px] text-app-foreground"
      style={{
        gridTemplateColumns: [showDate && "10.5rem", showJobId && "4rem", "minmax(0, 1fr)"]
          .filter(Boolean)
          .join(" "),
      }}
    >
      {showDate && (
        <time className="truncate text-app-foreground-muted" dateTime={log.created_at}>
          {log.created_at}
        </time>
      )}
      {showJobId && (
        <button
          type="button"
          title={log.job_run_id}
          aria-label={`Filter logs by job run ${shortId}`}
          aria-pressed={isActiveFilter}
          className={cn(
            "truncate text-left select-none hover:underline",
            isActiveFilter
              ? "text-app-accent"
              : "text-app-foreground-secondary hover:text-app-accent",
          )}
          onClick={() => applyJobRunFilter(log.job_run_id)}
        >
          {shortId}
        </button>
      )}
      <span className="min-w-0 wrap-break-word whitespace-pre-wrap">{log.content || " "}</span>
    </div>
  );
}
