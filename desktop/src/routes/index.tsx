import { createFileRoute, Link } from "@tanstack/react-router";
import { useLiveQuery } from "@tanstack/react-db";
import { workflowCollection } from "@/db/workflow";

export const Route = createFileRoute("/")({
  component: WorkflowsPage,
});

function WorkflowsPage() {
  const {
    data: workflows = [],
    isLoading,
    isError,
  } = useLiveQuery((q) =>
    q
      .from({ workflows: workflowCollection })
      .orderBy(({ workflows }) => workflows.id, "desc")
  );

  return (
    <div className="mx-auto max-w-4xl px-6 py-8">
      <div className="mb-6">
        <h2 className="text-lg font-medium">Workflows</h2>
      </div>

      {isLoading ? (
        <div className="py-12 text-center text-muted-foreground">
          Loading workflows...
        </div>
      ) : isError ? (
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-destructive">
          Something went wrong while loading workflows.
        </div>
      ) : workflows.length === 0 ? (
        <div className="rounded-lg border border-dashed p-12 text-center">
          <p className="text-muted-foreground">No workflows yet</p>
          <p className="mt-1 text-sm text-muted-foreground/70">
            Register a workflow using the CLI to get started
          </p>
        </div>
      ) : (
        <div className="space-y-2">
          {workflows.map((workflow) => (
            <Link
              key={workflow.id}
              to="/workflow/$workflowId"
              params={{ workflowId: workflow.id }}
              className="flex items-center justify-between rounded-lg border bg-card p-4 transition-colors hover:bg-accent/50 cursor-pointer group"
            >
              <div>
                <h3 className="font-medium group-hover:text-accent-foreground">
                  {workflow.name}
                </h3>
                <p className="mt-0.5 font-mono text-xs text-muted-foreground">
                  {workflow.id}
                </p>
              </div>
              <span className="text-muted-foreground group-hover:text-foreground transition-colors">
                →
              </span>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
