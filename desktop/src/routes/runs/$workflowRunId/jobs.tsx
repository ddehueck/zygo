import { eq } from "@tanstack/db";
import { useLiveQuery } from "@tanstack/react-db";
import { createFileRoute, Link, Outlet, useRouterState } from "@tanstack/react-router";

import { BreadcrumbHeaderLayout } from "@/components/layout/BreadcrumbHeaderLayout";
import { ScrollArea } from "@/components/ScrollArea";
import { Description, Heading, Text } from "@/components/Text";
import { jobRunsCollection, workflowRunsCollection, workflowsCollection } from "@/db/collections";
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
    query: (q) =>
      q
        .from({ workflowRun: workflowRunsCollection })
        .leftJoin(
          { workflow: workflowsCollection },
          ({ workflowRun, workflow }) => eq(workflowRun.workflow_id, workflow.id),
        )
        .where(({ workflowRun }) => eq(workflowRun.id, Number(workflowRunId)))
        .findOne(),
  });
  const jobsQuery = useLiveQuery({
    query: (q) => q.from({ jobRun: jobRunsCollection }),
  });

  const workflowRun = runsQuery.data?.workflowRun;
  const workflowName = runsQuery.data?.workflow?.name ?? "";
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
          Jobs for <span className="font-mono">{workflowName}</span>
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
  return (
    <BreadcrumbHeaderLayout>
      <ScrollArea>
        <main className="mx-auto w-full max-w-5xl px-6 py-10">{children}</main>
      </ScrollArea>
    </BreadcrumbHeaderLayout>
  );
}
