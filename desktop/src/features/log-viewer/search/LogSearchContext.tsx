import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";
import { eq } from "@tanstack/db";
import { useLiveQuery } from "@tanstack/react-db";

import { jobRunsCollection } from "@/db/collections";
import { LogSearchTokenValue } from "./log-search-token-value";
import { resolveJobRun } from "./use-log-search-suggestions";

type LogSearchContextValue = {
  value: LogSearchTokenValue;
  setValue: (value: LogSearchTokenValue) => void;
  contentQuery: string;
  /** Database job_runs.id used for query_logs, when a job filter is present. */
  jobRunId: number | null;
  /** Full public id for local log matching, when a job filter is present. */
  jobRunPublicId: string | null;
  applyJobRunFilter: (jobRunPublicId: string) => void;
  clear: () => void;
  isFiltering: boolean;
};

const LogSearchContext = createContext<LogSearchContextValue | null>(null);

export function LogSearchProvider({
  workflowRunId,
  children,
}: {
  workflowRunId: number;
  children: ReactNode;
}) {
  const [value, setValueRaw] = useState(() => new LogSearchTokenValue([]));

  const setValue = useCallback((next: LogSearchTokenValue) => {
    // Collapse typed/pasted duplicate job tokens — only one job filter is valid.
    setValueRaw(next.withSingleJobFilter());
  }, []);

  const jobsQuery = useLiveQuery({
    query: (q) =>
      q
        .from({ jobRun: jobRunsCollection })
        .where(({ jobRun }) => eq(jobRun.workflow_run_id, workflowRunId)),
  });

  const contentQuery = value.getContentQuery();
  const jobFilter = value.getFilterValues().find((filter) => filter.entity === "job_run") ?? null;
  const jobRunId = jobFilter?.job_run_id ?? null;
  const jobRunPublicId = jobFilter?.public_id ?? null;

  const applyJobRunFilter = useCallback(
    (publicId: string) => {
      const job = resolveJobRun(jobsQuery.data ?? [], publicId);
      if (job == null) return;
      setValueRaw((current) =>
        current.withJobRunFilter({
          entity: "job_run",
          job_run_id: job.id,
          public_id: job.public_id,
        }),
      );
    },
    [jobsQuery.data],
  );

  const clear = useCallback(() => {
    setValueRaw(new LogSearchTokenValue([]));
  }, []);

  const contextValue = useMemo(
    () => ({
      value,
      setValue,
      contentQuery,
      jobRunId,
      jobRunPublicId,
      applyJobRunFilter,
      clear,
      isFiltering: contentQuery.length > 0 || jobFilter != null,
    }),
    [
      value,
      setValue,
      contentQuery,
      jobRunId,
      jobRunPublicId,
      jobFilter,
      applyJobRunFilter,
      clear,
    ],
  );

  return <LogSearchContext.Provider value={contextValue}>{children}</LogSearchContext.Provider>;
}

export function useLogSearchContext() {
  const context = useContext(LogSearchContext);
  if (context == null) {
    throw new Error("useLogSearchContext must be used within LogSearchProvider");
  }
  return context;
}
