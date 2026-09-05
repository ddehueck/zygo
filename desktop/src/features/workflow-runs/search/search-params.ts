import { z } from "zod";

import type { WorkflowRunFilter } from "./types";

const filterSchema = z.discriminatedUnion("entity", [
  z.object({
    entity: z.literal("workflow"),
    id: z.string().min(1),
  }),
  z.object({
    entity: z.literal("tag"),
    name: z.string().min(1),
    value: z.string().optional(),
  }),
]) satisfies z.ZodType<WorkflowRunFilter>;

export const searchParamsSchema = z.object({
  filters: z.array(filterSchema).optional().catch(undefined),
});

export type SearchParams = z.output<typeof searchParamsSchema>;
