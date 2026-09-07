import { createFileRoute, Outlet, useRouterState } from "@tanstack/react-router";

import { WorkflowRunDetails } from "@/features/workflow-runs/components/WorkflowRunDetails";

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
  const { workflowRunId } = Route.useParams();
  const pathname = useRouterState({ select: (state) => state.location.pathname });

  return pathname === `/runs/${workflowRunId}` ? (
    <WorkflowRunDetails workflowRunId={workflowRunId} />
  ) : (
    <Outlet />
  );
}
