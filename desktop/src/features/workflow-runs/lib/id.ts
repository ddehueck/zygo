export function shortRunId(workflowRunId: string): string {
  return workflowRunId.slice(-4);
}
