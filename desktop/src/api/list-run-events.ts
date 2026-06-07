import { useQuery } from "@tanstack/react-query";
import { commands, ListRunEventsParams, Event } from "../bindings";

export function useListRunEvents(
  params: ListRunEventsParams,
  options?: {
    enabled?: boolean;
    refetchInterval?: number | false;
    onSuccess?: (data: Event[]) => void;
  }
) {
  return useQuery({
    queryKey: [
      "runEvents",
      params.run_id,
      params.sequence_number?.toString() ?? null,
      params.sort ?? null,
      params.limit,
    ],
    queryFn: async (): Promise<Event[]> => {
      const result = await commands.listRunEvents(params);
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
