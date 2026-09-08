import { createFileRoute } from "@tanstack/react-router";

import { BreadcrumbHeaderLayout } from "@/components/layout/BreadcrumbHeaderLayout";
import { ScrollArea } from "@/components/ScrollArea";
import { WorkflowRunDetails } from "@/features/workflow-runs/components/WorkflowRunDetails";

export const Route = createFileRoute("/runs/$workflowRunId/")({
  beforeLoad: () => ({ breadcrumb: undefined }),
  component: WorkflowRunIndexRoute,
});

function WorkflowRunIndexRoute() {
  const { workflowRunId } = Route.useParams();

  return (
    <BreadcrumbHeaderLayout>
      <ScrollArea>
        <WorkflowRunDetails workflowRunId={workflowRunId} />
      </ScrollArea>
    </BreadcrumbHeaderLayout>
  );
}
