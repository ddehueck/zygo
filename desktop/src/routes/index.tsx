import { createFileRoute } from "@tanstack/react-router";
import { BreadcrumbHeaderLayout } from "../components/layout/BreadcrumbHeaderLayout";
import { WorkflowRunList } from "../features/workflow-runs/components/WorkflowRunList";
import { useWorkflowRunsListData } from "../features/workflow-runs/hooks/use-workflow-runs-list-data";

export const Route = createFileRoute("/")({
  beforeLoad: () => ({
    breadcrumb: {
      label: "Workflow Runs",
      link: "/",
    },
  }),
  component: IndexRoute,
});

function IndexRoute() {
  const { data: runs, isLoading, isError, status } = useWorkflowRunsListData();

  return (
    <BreadcrumbHeaderLayout>
      <main className="flex w-full flex-col gap-6">
        {isLoading && <p className="px-6 text-app-foreground-muted">Loading workflow runs…</p>}
        {isError && (
          <p
            className="mx-6 rounded-md border border-app-danger/30 bg-app-danger/10 p-4 text-app-danger"
            role="alert"
          >
            Unable to load workflow runs (status: {status}).
          </p>
        )}

        {runs.length > 0 ? (
          <WorkflowRunList runs={runs} />
        ) : (
          !isLoading &&
          !isError && (
            <p className="mx-6 rounded-md border border-dashed border-app-border p-8 text-center text-app-foreground-muted">
              No workflow runs loaded.
            </p>
          )
        )}
      </main>
    </BreadcrumbHeaderLayout>
  );
}
