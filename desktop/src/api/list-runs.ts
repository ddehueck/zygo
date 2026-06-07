import { useQuery } from "@tanstack/react-query";
import { commands, ListRunsParams, Run } from "../bindings";

export function useListRuns(
  params: ListRunsParams,
  options?: {
    enabled?: boolean;
    refetchInterval?: number | false;
    onSuccess?: (data: Run[]) => void;
  }
) {
  return useQuery({
    queryKey: [
      "runs",
      params.workflow_id,
      params.run_id ?? null,
      params.sort ?? null,
      params.limit,
    ],
    queryFn: async (): Promise<Run[]> => {
      const result = await commands.listRuns(params);
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
