import { createFileRoute, Outlet } from "@tanstack/react-router";

export const Route = createFileRoute("/workflows/$workflowId/runs")({
  beforeLoad: () => ({ breadcrumb: undefined }),
  component: WorkflowRunsRoute,
});

function WorkflowRunsRoute() {
  return <Outlet />;
}
