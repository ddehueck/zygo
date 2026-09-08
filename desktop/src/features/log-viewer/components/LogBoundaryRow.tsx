import { StatusIcon } from "@/components/icons";
import { Text } from "@/components/Text";

const BOUNDARY_COPY = {
  start: "You're at the beginning of the log history",
  end: "You're at the end of the log history",
} as const;

export function LogBoundaryRow({
  edge,
  isFollowing = false,
}: {
  edge: "start" | "end";
  isFollowing?: boolean;
}) {
  const waitingForLogs = edge === "end" && isFollowing;

  return (
    <div className="flex items-center justify-center gap-1.5 border-b border-app-border/25 bg-app-bg-base/45 px-3 py-1.5">
      {waitingForLogs ? (
        <StatusIcon
          status="in-progress"
          className="size-3 shrink-0 text-app-success"
          aria-hidden
        />
      ) : null}
      <Text size="small" variant="muted" className="text-xs opacity-60">
        {waitingForLogs ? "Waiting for new logs…" : BOUNDARY_COPY[edge]}
      </Text>
    </div>
  );
}
