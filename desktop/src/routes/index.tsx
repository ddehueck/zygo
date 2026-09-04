import { createFileRoute } from "@tanstack/react-router";
import { BreadcrumbHeaderLayout } from "../components/layout/BreadcrumbHeaderLayout";
import { useWorkflowRunsListData } from "../features/workflow-runs/hooks/use-workflow-runs-list-data";
import { WorkflowRunSearch } from "@/features/workflow-runs/search/WorkflowRunSearch";
import { WorkflowRunList } from "@/features/workflow-runs/components/WorkflowRunList";

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
      {isLoading && <p className="px-6 text-app-foreground-muted">Loading workflow runs…</p>}
      {isError && (
        <p className="mx-6 p-4 text-app-danger" role="alert">
          Unable to load workflow runs (status: {status}).
        </p>
      )}

      {runs.length > 0 ? (
        <>
          <WorkflowRunSearch />
          <WorkflowRunList runs={runs} />
        </>
      ) : (
        !isLoading &&
        !isError && (
          <p className="mx-6 p-8 text-center text-app-foreground-muted">No workflow runs loaded.</p>
        )
      )}
    </BreadcrumbHeaderLayout>
  );
}
