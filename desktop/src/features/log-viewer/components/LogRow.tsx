import type { Log } from "@/bindings";

export const logRowClassName =
  "grid grid-cols-[4rem_10.5rem_11rem_minmax(0,1fr)] items-start gap-3 border-b border-app-border/60 px-3 py-1 font-mono text-xs leading-[18px] text-app-foreground";

export const logContentClassName = "min-w-0 break-words whitespace-pre-wrap";

export function LogRow({ log }: { log: Log }) {
  return (
    <div className={logRowClassName}>
      <span className="text-right text-app-foreground-muted tabular-nums">{log.id}</span>
      <time className="truncate text-app-foreground-muted" dateTime={log.created_at}>
        {log.created_at}
      </time>
      <span className="truncate text-app-foreground-secondary" title={log.job_run_id}>
        {log.job_run_id}
      </span>
      <span className={logContentClassName}>{log.content || " "}</span>
    </div>
  );
}
