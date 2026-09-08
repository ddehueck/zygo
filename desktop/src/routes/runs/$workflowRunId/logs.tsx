import { eq, useLiveQuery } from "@tanstack/react-db";
import { createFileRoute, Link } from "@tanstack/react-router";

import { Description, Heading, Text } from "@/components/Text";
import { workflowRunsCollection } from "@/db/collections";
import { LogViewer } from "@/features/log-viewer/components/LogViewer";
import { isPositiveInteger } from "@/lib/integer";

export const Route = createFileRoute("/runs/$workflowRunId/logs")({
  beforeLoad: ({ params }) => ({
    breadcrumb: {
      label: "Logs",
      link: `/runs/${params.workflowRunId}/logs`,
    },
  }),
  component: WorkflowRunLogsRoute,
});

function WorkflowRunLogsRoute() {
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
      <LogsPageShell>
        <Text variant="muted">Loading workflow run…</Text>
      </LogsPageShell>
    );
  }

  if (runsQuery.isError) {
    return (
      <LogsPageShell>
        <Text role="alert" size="small" variant="danger">
          Unable to load this workflow run.
        </Text>
      </LogsPageShell>
    );
  }

  if (!isPositiveInteger(numericWorkflowRunId) || !workflowRun) {
    return (
      <LogsPageShell>
        <Heading size="medium">Workflow run not found</Heading>
        <Description className="mt-2">The requested run may have been removed.</Description>
        <Link
          to="/"
          className="mt-5 inline-block text-sm font-medium text-app-accent hover:underline"
        >
          Back to workflow runs
        </Link>
      </LogsPageShell>
    );
  }

  return <LogViewer workflowRunId={numericWorkflowRunId} />;
}

function LogsPageShell({ children }: { children: React.ReactNode }) {
  return <main className="mx-auto w-full max-w-5xl px-6 py-10">{children}</main>;
}
