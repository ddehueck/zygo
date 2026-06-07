import { useMemo } from "react";
import { eq, useLiveQuery } from "@tanstack/react-db";
import { runEventsCollection } from "@/db/run-events";
import {
  RunEventsStore,
  createRunEventsStore,
  type JobState,
  type ChannelState,
  type JobExecution,
} from "../lib/run-events-store";

export type { JobState, ChannelState, JobExecution };

/**
 * Hook that provides access to a RunEventsStore for a specific run.
 * The store is memoized and only recreated when events change.
 */
export function useRunEventsStore(runId: string): {
  store: RunEventsStore | undefined;
  isLoading: boolean;
  isError: boolean;
} {
  const {
    data: events,
    isLoading,
    isError,
  } = useLiveQuery(
    (q) =>
      q
        .from({ events: runEventsCollection })
        .where(({ events }) => eq(events.workflow_run_id, runId))
        .orderBy(({ events }) => events.sequence_number, "asc"),
    [runId]
  );

  const store = useMemo(() => {
    if (!events) return undefined;
    return createRunEventsStore(events);
  }, [events]);

  return { store, isLoading, isError };
}

/**
 * Hook to get job state from the events store
 */
export function useJobStateFromStore({
  runId,
  jobId,
}: {
  runId: string;
  jobId: string;
}): {
  data: JobState | undefined;
  isLoading: boolean;
  isError: boolean;
} {
  const { store, isLoading, isError } = useRunEventsStore(runId);

  const data = useMemo(() => {
    if (!store) return undefined;
    return store.getJobState(jobId);
  }, [store, jobId]);

  return { data, isLoading, isError };
}

/**
 * Hook to get channel state from the events store
 */
export function useChannelStateFromStore({
  runId,
  channelId,
}: {
  runId: string;
  channelId: string;
}): {
  data: ChannelState | undefined;
  isLoading: boolean;
  isError: boolean;
} {
  const { store, isLoading, isError } = useRunEventsStore(runId);

  const data = useMemo(() => {
    if (!store) return undefined;
    return store.getChannelState(channelId);
  }, [store, channelId]);

  return { data, isLoading, isError };
}

/**
 * Hook to get job executions for a specific job
 */
export function useJobExecutions({
  runId,
  jobId,
}: {
  runId: string;
  jobId: string;
}): {
  data: JobExecution[] | undefined;
  isLoading: boolean;
  isError: boolean;
} {
  const { store, isLoading, isError } = useRunEventsStore(runId);

  const data = useMemo(() => {
    if (!store) return undefined;
    return store.getJobExecutions(jobId);
  }, [store, jobId]);

  return { data, isLoading, isError };
}
