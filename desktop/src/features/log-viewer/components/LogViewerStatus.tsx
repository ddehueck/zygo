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
        {error
          ? "Unable to load some logs"
          : isFetchingPreviousPage
            ? "Loading older logs…"
            : isFollowing
              ? "Following live output"
              : hasPreviousPage
                ? "Scroll to the top to load older logs"
                : "Viewing saved output"}
      </Text>
    </div>
  );
}
