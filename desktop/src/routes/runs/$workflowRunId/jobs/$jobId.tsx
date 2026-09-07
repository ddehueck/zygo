import { createFileRoute, Link } from "@tanstack/react-router";
import { useLiveQuery } from "@tanstack/react-db";
import { eq } from "@tanstack/db";

import {
  dataReferencesCollection,
  jobRunsCollection,
  logsCollection,
  workflowRunsCollection,
} from "@/db/collections";
import { Description, Heading, Text } from "@/components/Text";
import { useWatchLogs } from "@/hooks/use-watch-logs";

export const Route = createFileRoute("/runs/$workflowRunId/jobs/$jobId")({
  beforeLoad: ({ params }) => ({
    breadcrumb: {
      label: `Job (${params.jobId})`,
      link: `/runs/${params.workflowRunId}/jobs/${params.jobId}`,
    },
  }),
  component: JobRoute,
});

function JobRoute() {
  const { workflowRunId, jobId } = Route.useParams();

  return (
    <main className="mx-auto w-full max-w-5xl px-6 py-10">
      <Heading size="medium">Workflow logs</Heading>
      <Description className="mt-2">All jobs in run {workflowRunId}</Description>
      <JobLogs workflowRunId={workflowRunId} />
      <JobDataReferences workflowRunId={workflowRunId} jobRunId={jobId} />
      <Link
        to="/runs/$workflowRunId"
        params={{ workflowRunId }}
        className="mt-6 inline-block text-sm font-medium text-app-accent hover:underline"
      >
        Back to run overview
      </Link>
    </main>
  );
}

function JobDataReferences({
  workflowRunId,
  jobRunId,
}: {
  workflowRunId: string;
  jobRunId: string;
}) {
  const runsQuery = useLiveQuery({
    query: (q) => q.from({ workflowRun: workflowRunsCollection }),
  });
  const jobsQuery = useLiveQuery({
    query: (q) => q.from({ jobRun: jobRunsCollection }),
  });

  const referencesQuery = useLiveQuery({
    query: (q) => q.from({ reference: dataReferencesCollection }),
  });

  const workflowRun = runsQuery.data.find((run) => String(run.id) === workflowRunId);
  const jobRun = jobsQuery.data.find(
    (job) => String(job.id) === jobRunId && job.workflow_run_id === workflowRun?.id,
  );
  const references = referencesQuery.data
    .filter(
      (reference) =>
        reference.workflow_run_id === workflowRun?.id && reference.job_run_id === jobRun?.id,
    )
    .sort((a, b) => a.created_at.localeCompare(b.created_at));

  return (
    <section aria-label="Data references" className="mt-8">
      <Heading size="medium">Data references</Heading>
      <Description className="mt-2">References inserted during this job.</Description>
      {runsQuery.isError || jobsQuery.isError || referencesQuery.isError ? (
        <Text role="alert" variant="danger" size="small" className="mt-4 block">
          Unable to load data references.
        </Text>
      ) : runsQuery.isLoading || jobsQuery.isLoading || referencesQuery.isLoading ? (
        <Text variant="muted" size="small" className="mt-4 block">
          Loading data references…
        </Text>
      ) : references.length === 0 ? (
        <Text variant="muted" size="small" className="mt-4 block">
          No data references were inserted during this job.
        </Text>
      ) : (
        <ul className="mt-4 divide-y divide-app-border rounded-lg border border-app-border">
          {references.map((reference) => (
            <li key={reference.id} className="px-4 py-3">
              <Text size="small" className="block break-all">
                {reference.uri}
              </Text>
              <Description className="mt-1">
                Created {reference.created_at}
                {reference.is_replay ? " · Replayed" : ""}
              </Description>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function JobLogs({ workflowRunId }: { workflowRunId: string }) {
  const id = Number(workflowRunId);

  useWatchLogs({ workflowRunId: id });

  const logsQuery = useLiveQuery({
    query: (q) =>
      q
        .from({ logs: logsCollection })
        .where(({ logs }) => eq(logs.workflow_run_id, id))
        .orderBy(({ logs }) => logs.id, "asc"),
  });

  const contents = logsQuery.data.map((log) => log.content).join("");

  // todo a table component with virtualization + pretextjs for height computation
  return (
    <section aria-label="Workflow logs" className="mt-6">
      {logsQuery.isError && (
        <Text role="alert" size="small" variant="danger" className="mb-3 block">
          Unable to load workflow logs.
        </Text>
      )}
      <pre className="max-h-[65vh] min-h-64 overflow-auto rounded-lg border border-app-border p-4">
        <Text size="small">
          {contents || (logsQuery.isLoading ? "Loading logs…" : "No logs recorded.")}
        </Text>
      </pre>
    </section>
  );
}
