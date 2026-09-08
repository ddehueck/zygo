import type { Log } from "@/bindings";
import { shortRunId } from "@/features/workflow-runs/lib/id";

export const logRowClassName =
  "log-viewer-grid grid items-start gap-3 border-b border-app-border/60 px-3 py-1 font-mono text-xs leading-[18px] text-app-foreground";

export const logContentClassName = "min-w-0 break-words whitespace-pre-wrap";

export function logGridTemplateColumns(showDate: boolean, showJobId: boolean) {
  return `4rem ${showDate ? "10.5rem" : "0px"} ${showJobId ? "4rem" : "0px"} minmax(0, 1fr)`;
}

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
      className={logRowClassName}
      style={{ gridTemplateColumns: logGridTemplateColumns(showDate, showJobId) }}
    >
      <span className="text-left text-app-foreground-muted tabular-nums">{log.id}</span>
      {showDate ? (
        <time className="truncate text-app-foreground-muted" dateTime={log.created_at}>
          {log.created_at}
        </time>
      ) : (
        <span aria-hidden />
      )}
      {showJobId ? (
        <span className="truncate text-app-foreground-secondary" title={log.job_run_id}>
          {shortRunId(log.job_run_id)}
        </span>
      ) : (
        <span aria-hidden />
      )}
      <span className={logContentClassName}>{log.content || " "}</span>
    </div>
  );
}
