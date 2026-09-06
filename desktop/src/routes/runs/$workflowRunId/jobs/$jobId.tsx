import { createFileRoute, Link } from "@tanstack/react-router";

export const Route = createFileRoute("/runs/$workflowRunId/jobs/$jobId")({
  beforeLoad: ({ params }) => ({
    breadcrumb: {
      label: `Job (${params.jobId.slice(-4)})`,
      link: `/runs/${params.workflowRunId}/jobs/${params.jobId}`,
    },
  }),
  component: JobRoute,
});

function JobRoute() {
  const { workflowRunId, jobId } = Route.useParams();

  return (
    <main className="mx-auto w-full max-w-5xl px-6 py-10">
      <h1 className="text-xl font-semibold text-app-foreground">Job details</h1>
      <p className="mt-2 text-sm text-app-foreground-muted">Job detail is not implemented yet.</p>
      <p className="mt-4 font-mono text-xs text-app-foreground-muted">
        {workflowRunId} / {jobId}
      </p>
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
