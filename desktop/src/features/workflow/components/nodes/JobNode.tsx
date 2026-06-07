import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import { useJobState, type JobState } from "@/features/run/hooks/useJobState";
import {
  StatusIndicator,
  type StatusVariant,
} from "@/features/run/components/StatusIndicator";

type JobStatus = JobState["status"];

export type JobNodeData = {
  label: string;
  jobId: string;
  hasIncoming?: boolean;
  hasOutgoing?: boolean;
  runId?: string;
};

export type JobNodeType = Node<JobNodeData, "job">;

function mapJobStatusToVariant(status: JobStatus): StatusVariant {
  switch (status) {
    case "idle":
      return "idle";
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

function JobNodeStatus({ runId, jobId }: { runId: string; jobId: string }) {
  const { data } = useJobState({ runId, jobId });

  if (!data || data.status === "idle") {
    return null;
  }

  const variant = mapJobStatusToVariant(data.status);
  const count =
    data.runningCount > 0
      ? data.runningCount
      : data.pendingCount > 0
      ? data.pendingCount
      : undefined;

  return <StatusIndicator variant={variant} count={count} />;
}

export function JobNode({ data }: NodeProps<JobNodeType>) {
  return (
    <div className="group relative flex min-w-[140px] items-center gap-2 rounded-lg border border-primary/30 bg-card px-3 py-2.5 shadow-sm transition-all hover:border-primary/50 hover:shadow-md">
      {data.hasIncoming && (
        <Handle
          id="top"
          type="target"
          position={Position.Top}
          className="size-2! border! border-primary! bg-card! transition-all group-hover:bg-primary!"
        />
      )}
      <div className="flex size-7 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="currentColor"
          className="size-4"
        >
          <path
            fillRule="evenodd"
            d="M7.84 1.804A1 1 0 0 1 8.82 1h2.36a1 1 0 0 1 .98.804l.331 1.652a6.993 6.993 0 0 1 1.929 1.115l1.598-.54a1 1 0 0 1 1.186.447l1.18 2.044a1 1 0 0 1-.205 1.251l-1.267 1.113a7.047 7.047 0 0 1 0 2.228l1.267 1.113a1 1 0 0 1 .206 1.25l-1.18 2.045a1 1 0 0 1-1.187.447l-1.598-.54a6.993 6.993 0 0 1-1.929 1.115l-.33 1.652a1 1 0 0 1-.98.804H8.82a1 1 0 0 1-.98-.804l-.331-1.652a6.993 6.993 0 0 1-1.929-1.115l-1.598.54a1 1 0 0 1-1.186-.447l-1.18-2.044a1 1 0 0 1 .205-1.251l1.267-1.114a7.05 7.05 0 0 1 0-2.227L1.821 7.773a1 1 0 0 1-.206-1.25l1.18-2.045a1 1 0 0 1 1.187-.447l1.598.54A6.993 6.993 0 0 1 7.51 3.456l.33-1.652ZM10 13a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"
            clipRule="evenodd"
          />
        </svg>
      </div>
      <div className="flex flex-col">
        <span className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
          Job
        </span>
        <span className="font-sans text-sm font-medium text-foreground">
          {data.label}
        </span>
      </div>
      {data.runId && <JobNodeStatus runId={data.runId} jobId={data.jobId} />}
      {data.hasOutgoing && (
        <Handle
          id="bottom"
          type="source"
          position={Position.Bottom}
          className="size-2! border! border-primary! bg-card! transition-all group-hover:bg-primary!"
        />
      )}
    </div>
  );
}
