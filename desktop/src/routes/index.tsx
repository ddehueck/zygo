import { useLiveQuery } from "@tanstack/react-db";
import { createFileRoute } from "@tanstack/react-router";
import { useWorkflowRunsCollection } from "../db/workflow-run-summaries";

export const Route = createFileRoute("/")({
  component: IndexRoute,
});

function IndexRoute() {
  const workflowRuns = useWorkflowRunsCollection();
  const {
    data: summaries,
    isLoading,
    isError,
    status,
  } = useLiveQuery({
    query: (q) =>
      q
        .from({ summary: workflowRuns })
        .orderBy(({ summary }) => summary.created_at, "desc"),
  });

  return (
    <main className="container">
      <h1>Workflow run summaries</h1>

      {isLoading && <p>Loading summaries...</p>}
      {isError && (
        <p role="alert">Unable to load summaries (status: {status}).</p>
      )}

      {summaries.length > 0 ? (
        <ul>
          {summaries.map((summary) => (
            <li key={summary.workflow_run_id}>
              <pre>{JSON.stringify(summary, null, 2)}</pre>
            </li>
          ))}
        </ul>
      ) : (
        !isLoading && !isError && <p>No summaries loaded.</p>
      )}
    </main>
  );
}
