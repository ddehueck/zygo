import { LogViewport } from "./LogViewport";

export function LogViewer({ workflowRunId }: { workflowRunId: number }) {
  // TODO: Add search
  return <LogViewport key={workflowRunId} workflowRunId={workflowRunId} />;
}
