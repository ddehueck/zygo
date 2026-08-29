import { useState } from "react";
import { createFileRoute } from "@tanstack/react-router";
import { Button } from "../components/Button";
import { commands, type WorkflowRunSummary } from "../bindings";

export const Route = createFileRoute("/")({
  component: IndexRoute,
});

function IndexRoute() {
  const [summaries, setSummaries] = useState<WorkflowRunSummary[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function loadSummaries() {
    setIsLoading(true);
    setError(null);

    try {
      const result = await commands.listWorkflowRunSummaries({
        cursor: null,
        limit: 100,
      });

      if (result.status === "error") {
        throw new Error(result.error);
      }

      setSummaries(result.data.summaries);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <main className="container">
      <h1>Workflow run summaries</h1>

      <Button
        variant="primary"
        isDisabled={isLoading}
        onPress={() => void loadSummaries()}
      >
        {isLoading ? "Loading..." : "Load summaries"}
      </Button>

      {error && <p role="alert">Unable to load summaries: {error}</p>}

      {summaries.length > 0 ? (
        <ul>
          {summaries.map((summary) => (
            <li key={summary.workflow_run_id}>
              <pre>{JSON.stringify(summary, null, 2)}</pre>
            </li>
          ))}
        </ul>
      ) : (
        <p>No summaries loaded.</p>
      )}
    </main>
  );
}
