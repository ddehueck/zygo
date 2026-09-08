import { createFileRoute, Outlet } from "@tanstack/react-router";

export const Route = createFileRoute("/runs")({
  component: RouteComponent,
  beforeLoad: () => ({
    breadcrumb: {
      label: "Workflow Runs",
      link: "/",
    },
  }),
});

function RouteComponent() {
  return <Outlet />;
}
