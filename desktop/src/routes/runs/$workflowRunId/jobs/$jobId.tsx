import { createFileRoute, Link } from "@tanstack/react-router";
import { useLiveQuery } from "@tanstack/react-db";
import { Channel, Resource } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import { commands, type LogBatch } from "@/bindings";
import { dataReferencesCollection } from "@/db/collections";
import { upsertCollectionItem } from "@/db/collection-helpers";
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
  const [loadError, setLoadError] = useState<string | null>(null);
  useEffect(() => {
    let disposed = false;
    void commands
      .loadDataReferences({ workflow_run_id: workflowRunId, job_run_id: jobRunId })
      .then((result) => {
        if (disposed) return;
        if (result.status === "error") {
          setLoadError(result.error.message);
          return;
        }
        for (const reference of result.data) {
          upsertCollectionItem(dataReferencesCollection, reference);
        }
      })
      .catch((error: unknown) => {
        if (!disposed) setLoadError(String(error));
      });
    return () => {
      disposed = true;
    };
  }, [workflowRunId, jobRunId]);

  const referencesQuery = useLiveQuery({
    query: (q) => q.from({ reference: dataReferencesCollection }),
  });
  const references = referencesQuery.data
    .filter(
      (reference) =>
        reference.workflow_run_id === workflowRunId && reference.job_run_id === jobRunId,
    )
    .sort((a, b) => a.inserted_at.localeCompare(b.inserted_at));

  return (
    <section aria-label="Data references" className="mt-8">
      <Heading size="medium">Data references</Heading>
      <Description className="mt-2">References inserted during this job.</Description>
      {loadError ? (
        <Text role="alert" variant="danger" size="small" className="mt-4 block">
          Unable to load data references: {loadError}
        </Text>
      ) : referencesQuery.isLoading ? (
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
                Version {reference.version}
                {reference.is_replay ? " · Replayed" : ""}
              </Description>
            </li>
          ))}
        </ul>
      )}
    </section>
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
