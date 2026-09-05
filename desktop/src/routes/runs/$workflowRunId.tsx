import { createFileRoute } from "@tanstack/react-router";

import { WorkflowRunDetails } from "@/features/workflow-runs/components/WorkflowRunDetails";
import { shortRunId } from "@/features/workflow-runs/lib/id";

export const Route = createFileRoute("/runs/$workflowRunId")({
  beforeLoad: ({ params }) => ({
    breadcrumb: {
      label: `Run (${shortRunId(params.workflowRunId)})`,
      link: `/runs/${params.workflowRunId}`,
    },
  }),
  component: WorkflowRunRoute,
});

function WorkflowRunRoute() {
  const { workflowRunId } = Route.useParams();
  return <WorkflowRunDetails workflowRunId={workflowRunId} />;
}
