import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import { useChannelStatus } from "@/features/run/hooks/useChannelStatus";

export type ChannelNodeData = {
  label: string;
  channelId: string;
  hasIncoming?: boolean;
  hasOutgoing?: boolean;
  runId?: string;
};

export type ChannelNodeType = Node<ChannelNodeData, "channel">;

function ChannelItemCount({
  runId,
  channelId,
}: {
  runId: string;
  channelId: string;
}) {
  const { data } = useChannelStatus({ runId, channelId });

  if (!data || data.itemCount === 0) {
    return null;
  }

  return (
    <span className="flex items-center justify-center rounded-full bg-chart-2/20 px-1.5 py-0.5 text-xs font-medium text-chart-2">
      {data.itemCount}
    </span>
  );
}

export function ChannelNode({ data }: NodeProps<ChannelNodeType>) {
  return (
    <div className="group relative flex min-w-[140px] items-center gap-2 rounded-lg border border-chart-2/30 bg-card px-3 py-2.5 shadow-sm transition-all hover:border-chart-2/50 hover:shadow-md">
      {data.hasIncoming && (
        <Handle
          id="top"
          type="target"
          position={Position.Top}
          className="size-2! border! border-chart-2! bg-card! transition-all group-hover:bg-chart-2!"
        />
      )}
      <div className="flex size-7 shrink-0 items-center justify-center rounded-md bg-chart-2/10 text-chart-2">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="currentColor"
          className="size-4"
        >
          <path d="M12.5 2.75a.75.75 0 0 0-1.5 0v14.5a.75.75 0 0 0 1.5 0V2.75ZM8.5 5.5a.75.75 0 0 0-1.5 0v9a.75.75 0 0 0 1.5 0v-9ZM16.5 5.5a.75.75 0 0 0-1.5 0v9a.75.75 0 0 0 1.5 0v-9ZM4.5 8.25a.75.75 0 0 0-1.5 0v3.5a.75.75 0 0 0 1.5 0v-3.5Z" />
        </svg>
      </div>
      <div className="flex flex-col">
        <span className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
          Channel
        </span>
        <span className="font-sans text-sm font-medium text-foreground">
          {data.label}
        </span>
      </div>
      {data.runId && (
        <ChannelItemCount runId={data.runId} channelId={data.channelId} />
      )}
      {data.hasOutgoing && (
        <Handle
          id="bottom"
          type="source"
          position={Position.Bottom}
          className="size-2! border! border-chart-2! bg-card! transition-all group-hover:bg-chart-2!"
        />
      )}
    </div>
  );
}
