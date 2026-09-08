import { LogViewport } from "./LogViewport";

export function LogViewer({ workflowRunId }: { workflowRunId: number }) {
  return <LogViewport key={workflowRunId} workflowRunId={workflowRunId} />;
}
