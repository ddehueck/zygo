import { createFileRoute, Outlet } from "@tanstack/react-router";

export const Route = createFileRoute("/runs/$workflowRunId")({
  beforeLoad: ({ params }) => ({
    breadcrumb: {
      label: `Run (${params.workflowRunId})`,
      link: `/runs/${params.workflowRunId}`,
    },
  }),
  component: WorkflowRunRoute,
});

function WorkflowRunRoute() {
  return <Outlet />;
}
