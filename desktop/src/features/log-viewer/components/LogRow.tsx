import type { Log } from "@/bindings";
import { shortRunId } from "@/features/workflow-runs/lib/id";

export function LogRow({
  log,
  showDate = true,
  showJobId = true,
}: {
  log: Log;
  showDate?: boolean;
  showJobId?: boolean;
}) {
  return (
    <div
      className="log-viewer-grid grid items-start gap-3 border-b border-app-border/60 px-3 py-1 font-mono text-xs leading-[18px] text-app-foreground"
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
        <span className="truncate text-app-foreground-secondary" title={log.job_run_id}>
          {shortRunId(log.job_run_id)}
        </span>
      )}
      <span className="min-w-0 break-words whitespace-pre-wrap">{log.content || " "}</span>
    </div>
  );
}
