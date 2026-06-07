import {
  createFileRoute,
  Link,
  Outlet,
  useParams,
} from "@tanstack/react-router";
import { eq, useLiveQuery } from "@tanstack/react-db";
import { runsCollection } from "@/db/runs";
import { useRunState, type RunStatus } from "@/features/run/hooks/useRunState";
import { Blade, BladeHeader, BladeContent } from "@/components/layout/blade";

export const Route = createFileRoute("/workflow/$workflowId")({
  component: WorkflowLayout,
});

function getStatusColor(status: RunStatus) {
  switch (status) {
    case "completed":
      return "bg-emerald-500";
    case "failed":
      return "bg-red-500";
    case "running":
      return "bg-amber-500 animate-pulse";
    case "idle":
    default:
      return "bg-muted-foreground/50";
  }
}

function RunStatusIndicator({ runId }: { runId: string }) {
  const { data } = useRunState({ runId });
  const status = data?.state ?? "idle";

  return (
    <div
      className={`h-2 w-2 shrink-0 rounded-full ${getStatusColor(status)}`}
    />
  );
}

function WorkflowLayout() {
  const { workflowId } = Route.useParams();
  const params = useParams({ strict: false });
  const selectedRunId = "runId" in params ? params.runId : undefined;

  const {
    data: runs = [],
    isLoading,
    isError,
  } = useLiveQuery(
    (q) =>
      q
        .from({ runs: runsCollection })
        .where(({ runs }) => eq(runs.workflow_id, workflowId))
        .orderBy(({ runs }) => runs.created_at, "desc")
        .limit(25),
    [workflowId]
  );

  console.log(runs);

  return (
    <>
      <div className="flex h-full">
        {/* Left Blade - Runs Sidebar */}
        <Blade position="left" width="w-72">
          <BladeHeader>
            <h2 className="text-sm font-medium text-sidebar-foreground">
              Runs
            </h2>
            <p className="mt-0.5 truncate font-mono text-xs text-muted-foreground">
              {workflowId}
            </p>
          </BladeHeader>

          <BladeContent className="p-2">
            {isLoading ? (
              <div className="py-8 text-center text-sm text-muted-foreground">
                Loading runs...
              </div>
            ) : isError ? (
              <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive">
                Failed to load runs
              </div>
            ) : runs.length === 0 ? (
              <div className="py-8 text-center">
                <p className="text-sm text-muted-foreground">No runs yet</p>
                <p className="mt-1 text-xs text-muted-foreground/70">
                  Trigger a run using the CLI
                </p>
              </div>
            ) : (
              <div className="space-y-1">
                {runs.map((run) => {
                  const isSelected = selectedRunId === run.id;
                  return (
                    <Link
                      key={run.id}
                      to="/workflow/$workflowId/$runId"
                      params={{ workflowId, runId: run.id }}
                      className={`flex items-center gap-2 rounded-md px-3 py-2 text-sm ${
                        isSelected
                          ? "bg-sidebar-accent text-sidebar-accent-foreground"
                          : "text-sidebar-foreground hover:bg-sidebar-accent/50"
                      }`}
                    >
                      <RunStatusIndicator runId={run.id} />
                      <div className="min-w-0 flex-1">
                        <p className="truncate font-mono text-xs">{run.id}</p>
                        {run.created_at && (
                          <p className="mt-0.5 text-xs text-muted-foreground">
                            {new Date(run.created_at).toLocaleString()}
                          </p>
                        )}
                      </div>
                    </Link>
                  );
                })}
              </div>
            )}
          </BladeContent>
        </Blade>

        {/* Main Content Area */}
        <main className="flex flex-1 flex-col overflow-hidden">
          {/* Content */}
          <div className="flex-1 overflow-hidden">
            <Outlet />
          </div>
        </main>
      </div>
    </>
  );
}
