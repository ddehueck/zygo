import { Description, Text } from "@/components/Text";
import { useLogViewerData } from "../hooks/use-log-viewer-data";
import { LogViewport } from "./LogViewport";

export function LogViewer({ workflowRunId }: { workflowRunId: number }) {
  const query = useLogViewerData(workflowRunId);

  if (query.isLoading) {
    return (
      <div className="flex min-h-64 flex-1 items-center justify-center">
        <Text variant="muted">Loading workflow logs…</Text>
      </div>
    );
  }

  if (query.isError) {
    return (
      <div className="flex min-h-64 flex-1 items-center justify-center">
        <Text role="alert" size="small" variant="danger">
          Unable to load workflow logs.
        </Text>
      </div>
    );
  }

  if (query.logs.length === 0) {
    return (
      <div className="flex min-h-64 flex-1 items-center justify-center">
        <Description>No logs recorded for this workflow run.</Description>
      </div>
    );
  }

  return (
    <LogViewport
      logs={query.logs}
      hasNextPage={query.hasNextPage}
      isFetchingNextPage={query.isFetchingNextPage}
      fetchNextPage={query.fetchNextPage}
      watchError={query.watchError}
    />
  );
}
