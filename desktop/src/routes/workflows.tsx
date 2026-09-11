import { createFileRoute, Outlet } from "@tanstack/react-router";

export const Route = createFileRoute("/workflows")({
  beforeLoad: () => ({ breadcrumb: undefined }),
  component: WorkflowsRoute,
});

function WorkflowsRoute() {
  return <Outlet />;
}
