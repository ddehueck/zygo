import { createFileRoute, Outlet } from "@tanstack/react-router";

export const Route = createFileRoute("/workflows/$workflowId")({
  beforeLoad: () => ({ breadcrumb: undefined }),
  component: WorkflowRoute,
});

function WorkflowRoute() {
  return <Outlet />;
}
