import { eq, useLiveQuery } from "@tanstack/react-db";
import { createFileRoute, Link } from "@tanstack/react-router";

import { BreadcrumbHeaderLayout } from "@/components/layout/BreadcrumbHeaderLayout";
import { ScrollArea } from "@/components/ScrollArea";
import { Description, Heading, Text } from "@/components/Text";
import { workflowRunsCollection } from "@/db/collections";
import { DataViewer } from "@/features/data-viewer/components/DataViewer";
import { isPositiveInteger } from "@/lib/integer";

export const Route = createFileRoute("/runs/$workflowRunId/data")({
  beforeLoad: ({ params }) => ({
    breadcrumb: {
      label: "Data",
      link: `/runs/${params.workflowRunId}/data`,
    },
  }),
  component: WorkflowRunDataRoute,
});

function WorkflowRunDataRoute() {
  const { workflowRunId } = Route.useParams();
  const numericWorkflowRunId = Number(workflowRunId);

  const runsQuery = useLiveQuery({
    query: (q) =>
      q
        .from({ workflowRun: workflowRunsCollection })
        .where(({ workflowRun }) => eq(workflowRun.id, Number(workflowRunId)))
        .findOne(),
  });
  const workflowRun = runsQuery.data;

  if (runsQuery.isLoading) {
    return (
      <DataPageShell>
        <Text variant="muted">Loading workflow run…</Text>
      </DataPageShell>
    );
  }

  if (runsQuery.isError) {
    return (
      <DataPageShell>
        <Text role="alert" size="small" variant="danger">
          Unable to load this workflow run.
        </Text>
      </DataPageShell>
    );
  }

  if (!isPositiveInteger(numericWorkflowRunId) || !workflowRun) {
    return (
      <DataPageShell>
        <Heading size="medium">Workflow run not found</Heading>
        <Description className="mt-2">The requested run may have been removed.</Description>
        <Link
          to="/"
          className="mt-5 inline-block text-sm font-medium text-app-accent hover:underline"
        >
          Back to workflow runs
        </Link>
      </DataPageShell>
    );
  }

  return (
    <BreadcrumbHeaderLayout>
      <DataViewer workflowRunId={numericWorkflowRunId} workflowId={workflowRun.workflow_id} />
    </BreadcrumbHeaderLayout>
  );
}

function DataPageShell({ children }: { children: React.ReactNode }) {
  return (
    <BreadcrumbHeaderLayout>
      <ScrollArea>
        <main className="mx-auto w-full max-w-5xl px-6 py-10">{children}</main>
      </ScrollArea>
    </BreadcrumbHeaderLayout>
  );
}
