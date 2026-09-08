import { Text } from "@/components/Text";

export function LogViewerStatus({
  rowCount,
  isFollowing,
  isFetchingPreviousPage,
  hasPreviousPage,
  error,
}: {
  rowCount: number;
  isFollowing: boolean;
  isFetchingPreviousPage: boolean;
  hasPreviousPage: boolean;
  error: unknown;
}) {
  return (
    <div className="flex min-h-8 shrink-0 items-center justify-between gap-4 border-t border-app-border px-3 py-1.5">
      <Text size="small" variant="muted">
        {rowCount.toLocaleString()} loaded {rowCount === 1 ? "line" : "lines"}
      </Text>
      <Text size="small" variant={error ? "danger" : "muted"} role={error ? "alert" : undefined}>
        {statusMessage({ error, isFetchingPreviousPage, isFollowing, hasPreviousPage })}
      </Text>
    </div>
  );
}

function statusMessage({
  error,
  isFetchingPreviousPage,
  isFollowing,
  hasPreviousPage,
}: {
  error: unknown;
  isFetchingPreviousPage: boolean;
  isFollowing: boolean;
  hasPreviousPage: boolean;
}) {
  if (error) return "Unable to load some logs";
  if (isFetchingPreviousPage) return "Loading older logs…";
  if (isFollowing) return "Following live output";
  if (hasPreviousPage) return "Scroll to the top to load older logs";
  return "Viewing saved output";
}
