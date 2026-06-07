import { useQuery } from "@tanstack/react-query";
import { commands, ListWorkflowsParams, Workflow } from "../bindings";

export function useListWorkflows(
  params: ListWorkflowsParams,
  options?: {
    enabled?: boolean;
    refetchInterval?: number | false;
    onSuccess?: (data: Workflow[]) => void;
  }
) {
  return useQuery({
    queryKey: [
      "workflows",
      params.workflow_id ?? null,
      params.sort ?? null,
      params.limit,
    ],
    queryFn: async (): Promise<Workflow[]> => {
      const result = await commands.listWorkflows(params);
      if (result.status !== "ok") {
        throw new Error(result.error);
      }
      options?.onSuccess?.(result.data);
      return result.data;
    },
    enabled: options?.enabled,
    refetchInterval: options?.refetchInterval,
  });
}

