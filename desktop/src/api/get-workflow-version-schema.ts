import { useQuery } from "@tanstack/react-query";
import {
  commands,
  GetWorkflowVersionSchemaParams,
  WorkflowVersionSchema,
} from "../bindings";

export function useGetWorkflowVersionSchema(
  params: GetWorkflowVersionSchemaParams
) {
  return useQuery({
    queryKey: ["workflowVersionSchema", params.workflow_version_id],
    queryFn: async (): Promise<WorkflowVersionSchema> => {
      const result = await commands.getWorkflowVersionSchema(params);
      if (result.status === "ok") {
        return result.data;
      }
      throw new Error(result.error);
    },
    staleTime: 1000 * 60 * 60 * 24, // 24 hours
    gcTime: 1000 * 60 * 60 * 24, // 24 hours
  });
}
