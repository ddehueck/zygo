import { useLiveQuery } from "@tanstack/react-db";
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
    query: (q) => q.from({ workflowRun: workflowRunsCollection }),
  });
  const workflowRun = runsQuery.data.find((run) => run.id === numericWorkflowRunId);

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

  return (
    <main className="flex h-full min-h-[32rem] w-full flex-col px-4 py-4">
      <header className="mb-4 shrink-0">
        <Heading size="medium">Workflow logs</Heading>
        <Description className="mt-1">
          Live output for <span className="font-mono">{workflowRun.workflow_id}</span>
        </Description>
      </header>
      <LogViewer workflowRunId={numericWorkflowRunId} />
    </main>
  );
}

function LogsPageShell({ children }: { children: React.ReactNode }) {
  return <main className="mx-auto w-full max-w-5xl px-6 py-10">{children}</main>;
}
