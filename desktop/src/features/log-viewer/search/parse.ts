import { TokenSegment } from "react-aria-components/TokenField";
import { filterPrefix } from "@/components/search";
import { shortRunId } from "@/features/workflow-runs/lib/id";
import type { LogSearchFilter } from "./types";

const JOB_RUN_PREFIX = "job_run";

/** Chip label uses the short public id; value carries the numeric query id. */
export function toTokenSegment(filter: LogSearchFilter): TokenSegment<LogSearchFilter> {
  return {
    type: "token",
    text: `${filterPrefix(JOB_RUN_PREFIX)}${shortRunId(filter.public_id)}`,
    value: filter,
  };
}

export { getFilterValue } from "@/components/search";
