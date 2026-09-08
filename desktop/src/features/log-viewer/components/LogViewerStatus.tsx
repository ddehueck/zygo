import { Text } from "@/components/Text";

export function LogViewerStatus({
  rowCount,
  isFollowing,
  isFetchingOlder,
  hasOlder,
  watchError,
}: {
  rowCount: number;
  isFollowing: boolean;
  isFetchingOlder: boolean;
  hasOlder: boolean;
  watchError: unknown;
}) {
  return (
    <div className="flex min-h-8 shrink-0 items-center justify-between gap-4 border-t border-app-border px-3 py-1.5">
      <Text size="small" variant="muted">
        {rowCount.toLocaleString()} loaded {rowCount === 1 ? "line" : "lines"}
      </Text>
      <Text
        size="small"
        variant={watchError ? "danger" : "muted"}
        role={watchError ? "alert" : undefined}
      >
        {watchError
          ? "Live updates unavailable"
          : isFetchingOlder
            ? "Loading older logs…"
            : isFollowing
              ? "Following live output"
              : hasOlder
                ? "Scroll to the top to load older logs"
                : "Viewing saved output"}
      </Text>
    </div>
  );
}
