import { createFileRoute } from "@tanstack/react-router";

import { BreadcrumbHeaderLayout } from "@/components/layout/BreadcrumbHeaderLayout";
import { NotFound } from "@/components/NotFound";
import { ScrollArea } from "@/components/ScrollArea";
import { NewWorkflowRunPage } from "@/features/workflow-runs/components/NewWorkflowRunPage";
import { isPositiveInteger } from "@/lib/integer";

export const Route = createFileRoute("/workflows/$workflowId/runs/new")({
  beforeLoad: ({ params }) => ({
    breadcrumb: {
      label: "New run",
      link: `/workflows/${params.workflowId}/runs/new`,
    },
  }),
  component: NewWorkflowRunRoute,
});

function NewWorkflowRunRoute() {
  const { workflowId } = Route.useParams();
  const numericWorkflowId = Number(workflowId);

  return (
    <BreadcrumbHeaderLayout>
      <ScrollArea>
        {isPositiveInteger(numericWorkflowId) ? (
          <NewWorkflowRunPage workflowId={numericWorkflowId} />
        ) : (
          <NotFound
            title="Workflow not found"
            description="The workflow id in this URL is invalid."
          />
        )}
      </ScrollArea>
    </BreadcrumbHeaderLayout>
  );
}
