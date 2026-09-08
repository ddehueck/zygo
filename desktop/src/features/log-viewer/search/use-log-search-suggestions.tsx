import { eq } from "@tanstack/db";
import { useLiveQuery } from "@tanstack/react-db";

import { FILTER_DELIMITER, filterPrefix, getFilterValue } from "@/components/search";
import { jobRunsCollection } from "@/db/collections";
import { shortRunId } from "@/features/workflow-runs/lib/id";
import type { LogSearchSuggestion } from "./types";

const JOB_RUN_PREFIX = "job_run";

const defaultSuggestions: LogSearchSuggestion[] = [
  { id: "job-run-search-prefix", type: "prefix", text: filterPrefix(JOB_RUN_PREFIX) },
];

/**
 * Job-run suggestions for the current workflow run. Matches the typed search
 * string against full suggestion text (e.g. `@job_run:ab12`), so partial
 * prefixes like `@` or `@job` still keep matching chips visible.
 *
 * Job run filters are mutually exclusive — when one is already present, suggestions
 * are suppressed so a second job_run token cannot be added from the bar.
 */
export function useLogSearchSuggestions({
  workflowRunId,
  searchString,
  limit,
  hasJobFilter,
}: {
  workflowRunId: number;
  searchString: string;
  limit: number;
  hasJobFilter: boolean;
}) {
  const needle = searchString.trim().toLocaleLowerCase();
  // Only treat text after `:` as an id needle. While typing `@` / `@job_run`,
  // keep all candidates and filter by suggestion text below.
  const idNeedle = searchString.includes(FILTER_DELIMITER)
    ? getFilterValue(searchString).trim().toLocaleLowerCase()
    : "";

  const { data, isLoading, isError, status } = useLiveQuery({
    query: (q) =>
      q
        .from({ jobRun: jobRunsCollection })
        .where(({ jobRun }) => eq(jobRun.workflow_run_id, workflowRunId))
        .orderBy(({ jobRun }) => jobRun.id, "asc")
        .limit(200),
  });

  if (hasJobFilter) {
    return { suggestions: [] as LogSearchSuggestion[], isLoading, isError, status };
  }

  const jobSuggestions: LogSearchSuggestion[] = (data ?? [])
    .filter((jobRun) => {
      if (!idNeedle) return true;
      const publicId = jobRun.public_id.toLocaleLowerCase();
      const short = shortRunId(jobRun.public_id).toLocaleLowerCase();
      return publicId.includes(idNeedle) || short.includes(idNeedle);
    })
    .slice(0, limit)
    .map((jobRun) => ({
      id: `job_run:${jobRun.public_id}`,
      type: "token" as const,
      text: `${filterPrefix(JOB_RUN_PREFIX)}${shortRunId(jobRun.public_id)}`,
      value: {
        entity: "job_run" as const,
        job_run_id: jobRun.id,
        public_id: jobRun.public_id,
      },
    }));

  const suggestions: LogSearchSuggestion[] = [
    ...defaultSuggestions,
    ...jobSuggestions,
  ].filter(
    (item) => !needle || item.text.toLocaleLowerCase().includes(needle),
  );

  return { suggestions, isLoading, isError, status };
}

/** Resolve a typed short/long public id to a single job run when unambiguous. */
export function resolveJobRun(
  jobs: Array<{ id: number; public_id: string }>,
  filterId: string,
): { id: number; public_id: string } | null {
  const exact = jobs.find((job) => job.public_id === filterId);
  if (exact) return { id: exact.id, public_id: exact.public_id };

  const suffixMatches = jobs.filter((job) => job.public_id.endsWith(filterId));
  if (suffixMatches.length === 1) {
    return { id: suffixMatches[0].id, public_id: suffixMatches[0].public_id };
  }

  return null;
}
