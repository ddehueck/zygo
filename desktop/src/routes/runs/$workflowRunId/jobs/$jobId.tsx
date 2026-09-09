import { createFileRoute, Link } from "@tanstack/react-router";
import { useLiveQuery } from "@tanstack/react-db";

import {
  dataReferencesCollection,
  jobRunsCollection,
  workflowRunsCollection,
} from "@/db/collections";
import { Description, Heading, Text } from "@/components/Text";

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
      <Heading size="medium">Job details</Heading>
      <Description className="mt-2">Job {jobId}</Description>
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
        reference.workflow_run_id === workflowRun?.id && reference.source_job_run_id === jobRun?.id,
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
