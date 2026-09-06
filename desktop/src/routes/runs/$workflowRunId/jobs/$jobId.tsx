import { createFileRoute, Link } from "@tanstack/react-router";
import { Channel, Resource } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import { commands, type LogBatch } from "@/bindings";
import { Description, Heading, Text } from "@/components/Text";

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
      <Heading size="medium">Job logs</Heading>
      <Description className="mt-2">{jobId}</Description>
      <JobLogs key={jobId} jobRunId={jobId} />
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

function JobLogs({ jobRunId }: { jobRunId: string }) {
  const [contents, setContents] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let disposed = false;
    let subscription: Resource | undefined;
    const onBatch = new Channel<LogBatch>();
    onBatch.onmessage = (batch) => {
      if (disposed) return;
      if (batch.content) setContents((current) => current + batch.content);
      setError(batch.error);
    };

    const close = (resource: Resource) => {
      void resource.close().catch((error: unknown) => {
        console.error("Could not stop log watcher", error);
      });
    };

    void commands.watchLogs(jobRunId, onBatch).then(
      (result) => {
        if (result.status === "error") {
          if (!disposed) setError(result.error.message);
          return;
        }
        subscription = new Resource(result.data);
        if (disposed) close(subscription);
      },
      (error: unknown) => {
        if (!disposed) setError(String(error));
      },
    );

    return () => {
      disposed = true;
      if (subscription) close(subscription);
    };
  }, [jobRunId]);

  return (
    <section aria-label="Job logs" className="mt-6">
      {error && (
        <Text role="alert" size="small" variant="danger" className="mb-3 block">
          {error}
        </Text>
      )}
      <pre className="max-h-[65vh] min-h-64 overflow-auto rounded-lg border border-app-border p-4">
        <Text size="small">{contents || (error ? "" : "Waiting for logs…")}</Text>
      </pre>
    </section>
  );
}
