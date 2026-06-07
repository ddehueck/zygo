import { cn } from "@/lib/utils";
import type { JobExecution, JobExecutionStatus } from "../lib/run-events-store";
import {
  StatusIndicator,
  type StatusVariant,
} from "./StatusIndicator";

function mapExecutionStatusToVariant(status: JobExecutionStatus): StatusVariant {
  switch (status) {
    case "pending":
      return "pending";
    case "running":
      return "running";
    case "completed":
      return "success";
    case "errored":
      return "error";
  }
}

function formatTime(timestamp: string | null): string {
  if (!timestamp) return "—";
  try {
    const date = new Date(timestamp);
    return date.toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  } catch {
    return "—";
  }
}

function truncateKey(key: string, maxLength: number = 12): string {
  if (key.length <= maxLength) return key;
  return `${key.slice(0, maxLength)}…`;
}

type JobExecutionRowProps = {
  execution: JobExecution;
  className?: string;
};

export function JobExecutionRow({ execution, className }: JobExecutionRowProps) {
  const variant = mapExecutionStatusToVariant(execution.status);
  const startTime = formatTime(execution.startedAt);
  const endTime = execution.endedAt ? formatTime(execution.endedAt) : null;

  return (
    <div
      className={cn(
        "flex items-center gap-3 rounded-md border border-border/50 bg-card/50 px-3 py-2 text-sm",
        className
      )}
    >
      <StatusIndicator variant={variant} size="md" />
      <div className="min-w-0 flex-1">
        <p
          className="truncate font-mono text-xs text-foreground"
          title={execution.idempotencyKey}
        >
          {truncateKey(execution.idempotencyKey, 20)}
        </p>
        <p className="mt-0.5 text-xs text-muted-foreground">
          {startTime}
          {endTime && ` → ${endTime}`}
        </p>
      </div>
      <StatusLabel status={execution.status} />
    </div>
  );
}

function StatusLabel({ status }: { status: JobExecutionStatus }) {
  const labels: Record<JobExecutionStatus, string> = {
    pending: "Pending",
    running: "Running",
    completed: "Done",
    errored: "Failed",
  };

  const colorClasses: Record<JobExecutionStatus, string> = {
    pending: "text-muted-foreground",
    running: "text-blue-500",
    completed: "text-green-500",
    errored: "text-destructive",
  };

  return (
    <span className={cn("text-xs font-medium", colorClasses[status])}>
      {labels[status]}
    </span>
  );
}

