import { useLiveQuery } from "@tanstack/react-db";
import { createFileRoute, Link, Outlet, useRouterState } from "@tanstack/react-router";

import { Description, Heading, Text } from "@/components/Text";
import { jobRunsCollection, workflowRunsCollection } from "@/db/collections";
import { JobList } from "@/features/workflow-runs/components/JobList";

export const Route = createFileRoute("/runs/$workflowRunId/jobs")({
  beforeLoad: ({ params }) => ({
    breadcrumb: {
      label: "Jobs",
      link: `/runs/${params.workflowRunId}/jobs`,
    },
  }),
  component: JobsRoute,
});

function JobsRoute() {
  const { workflowRunId } = Route.useParams();
  const pathname = useRouterState({ select: (state) => state.location.pathname });

  return pathname === `/runs/${workflowRunId}/jobs` ? (
    <JobsListPage workflowRunId={workflowRunId} />
  ) : (
    <Outlet />
  );
}

function JobsListPage({ workflowRunId }: { workflowRunId: string }) {
  const runsQuery = useLiveQuery({
    query: (q) => q.from({ workflowRun: workflowRunsCollection }),
  });
  const jobsQuery = useLiveQuery({
    query: (q) => q.from({ jobRun: jobRunsCollection }),
  });

  const workflowRun = runsQuery.data.find((run) => String(run.id) === workflowRunId);
  const jobs = jobsQuery.data
    .filter((job) => job.workflow_run_id === workflowRun?.id)
    .sort((a, b) => a.created_at.localeCompare(b.created_at));

  if (runsQuery.isLoading || jobsQuery.isLoading) {
    return (
      <JobsPageShell>
        <Text size="medium" variant="muted">
          Loading jobs…
        </Text>
      </JobsPageShell>
    );
  }

  if (runsQuery.isError || jobsQuery.isError) {
    return (
      <JobsPageShell>
        <Text size="small" variant="danger" role="alert">
          Unable to load jobs for this workflow run.
        </Text>
      </JobsPageShell>
    );
  }

  if (!workflowRun) {
    return (
      <JobsPageShell>
        <Heading size="medium">Workflow run not found</Heading>
        <Description className="mt-2">The requested run may have been removed.</Description>
        <Link
          to="/"
          className="mt-5 inline-block text-sm font-medium text-app-accent hover:underline"
        >
          Back to workflow runs
        </Link>
      </JobsPageShell>
    );
  }

  return (
    <JobsPageShell>
      <header>
        <Heading size="medium">Jobs</Heading>
        <Description className="mt-1">
          Jobs for <span className="font-mono">{workflowRun.workflow_id}</span>
        </Description>
      </header>
      {jobs.length > 0 ? (
        <JobList jobs={jobs} />
      ) : (
        <Description className="mt-8">No jobs recorded.</Description>
      )}
    </JobsPageShell>
  );
}

function JobsPageShell({ children }: { children: React.ReactNode }) {
  return <main className="mx-auto w-full max-w-5xl px-6 py-10">{children}</main>;
}
