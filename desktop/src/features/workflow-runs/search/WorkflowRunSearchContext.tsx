import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";

import type { WorkflowRunListData } from "@/features/workflow-runs/hooks/use-workflow-runs-list-data";
import { toTokenSegment } from "./parse";
import type { WorkflowRunFilter } from "./types";
import { WorkflowSearchTokenValue } from "./workflow-run-search-token-value";

type WorkflowRunSearchContextValue = {
  value: WorkflowSearchTokenValue;
  setValue: (value: WorkflowSearchTokenValue) => void;
  activeFilters: WorkflowRunFilter[];
  applyFilters: (runs: WorkflowRunListData) => WorkflowRunListData;
};

const WorkflowRunSearchContext = createContext<WorkflowRunSearchContextValue | null>(null);

type WorkflowRunSearchProviderProps = {
  children: ReactNode;
  filters: WorkflowRunFilter[];
  onFiltersChange: (filters: WorkflowRunFilter[]) => void;
};

export function WorkflowRunSearchProvider({
  children,
  filters,
  onFiltersChange,
}: WorkflowRunSearchProviderProps) {
  const [value, setValue] = useUrlBackedTokenValue(filters, onFiltersChange);

  const activeFilters = filters;

  const contextValue = useMemo(
    () => ({
      value,
      setValue,
      activeFilters,
      applyFilters: (runs: WorkflowRunListData) =>
        runs.filter(({ workflowRun, tags }) =>
          activeFilters.every((filter) => {
            switch (filter.entity) {
              case "workflow":
                return workflowRun.workflow_id === filter.id;
              case "tag":
                return tags.some((tag) => tag.value === filter.value);
            }
          }),
        ),
    }),
    [activeFilters, setValue, value],
  );

  return (
    <WorkflowRunSearchContext.Provider value={contextValue}>
      {children}
    </WorkflowRunSearchContext.Provider>
  );
}

function useUrlBackedTokenValue(
  filters: WorkflowRunFilter[],
  onFiltersChange: (filters: WorkflowRunFilter[]) => void,
) {
  const [state, setState] = useState(() => ({
    filters,
    value: createTokenValue(filters),
  }));

  if (!areFiltersEqual(state.filters, filters)) {
    const value = areFiltersEqual(state.value.getFilterValues(), filters)
      ? state.value
      : createTokenValue(filters);
    setState({ filters, value });
  }

  const setValue = useCallback(
    (value: WorkflowSearchTokenValue) => {
      setState({ filters, value });

      const nextFilters = value.getFilterValues();
      if (!areFiltersEqual(filters, nextFilters)) onFiltersChange(nextFilters);
    },
    [filters, onFiltersChange],
  );

  return [state.value, setValue] as const;
}

function createTokenValue(filters: WorkflowRunFilter[]) {
  return new WorkflowSearchTokenValue(filters.map(toTokenSegment));
}

function areFiltersEqual(left: WorkflowRunFilter[], right: WorkflowRunFilter[]) {
  return (
    left.length === right.length &&
    left.every((filter, index) => {
      const other = right[index];

      switch (filter.entity) {
        case "workflow":
          return other.entity === "workflow" && filter.id === other.id;
        case "tag":
          return other.entity === "tag" && filter.value === other.value;
      }
    })
  );
}

export function useWorkflowRunSearch() {
  const context = useContext(WorkflowRunSearchContext);
  if (context == null) {
    throw new Error("useWorkflowRunSearch must be used within WorkflowRunSearchProvider");
  }
  return context;
}
