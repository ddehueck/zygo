import { createFileRoute } from "@tanstack/react-router";
import { useCallback } from "react";
import { BreadcrumbHeaderLayout } from "../components/layout/BreadcrumbHeaderLayout";
import { useWorkflowRunsListData } from "../features/workflow-runs/hooks/use-workflow-runs-list-data";
import { WorkflowRunSearch } from "@/features/workflow-runs/search/WorkflowRunSearch";
import { WorkflowRunList } from "@/features/workflow-runs/components/WorkflowRunList";
import { WorkflowRunSearchProvider } from "@/features/workflow-runs/search/WorkflowRunSearchContext";
import {
  type SearchParams,
  searchParamsSchema,
} from "@/features/workflow-runs/search/search-params";
import type { WorkflowRunFilter } from "@/features/workflow-runs/search/types";

export const Route = createFileRoute("/")({
  validateSearch: searchParamsSchema,
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
  const { filters = [] } = Route.useSearch();
  const navigate = Route.useNavigate();
  const setFilters = useCallback(
    (nextFilters: WorkflowRunFilter[]) => {
      void navigate({
        search: (previous: SearchParams) => ({
          ...previous,
          filters: nextFilters.length === 0 ? undefined : nextFilters,
        }),
        replace: true,
      });
    },
    [navigate],
  );

  return (
    <BreadcrumbHeaderLayout>
      {isLoading && <p className="px-6 text-app-foreground-muted">Loading workflow runs…</p>}
      {isError && (
        <p className="mx-6 p-4 text-app-danger" role="alert">
          Unable to load workflow runs (status: {status}).
        </p>
      )}

      {runs.length > 0 ? (
        <WorkflowRunSearchProvider filters={filters} onFiltersChange={setFilters}>
          <WorkflowRunSearch />
          <WorkflowRunList runs={runs} />
        </WorkflowRunSearchProvider>
      ) : (
        !isLoading &&
        !isError && (
          <p className="mx-6 p-8 text-center text-app-foreground-muted">No workflow runs loaded.</p>
        )
      )}
    </BreadcrumbHeaderLayout>
  );
}
