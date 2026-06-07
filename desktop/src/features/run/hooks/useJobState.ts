import { useJobStateFromStore, type JobState } from "./useRunEventsStore";

export type { JobState };

/**
 * Hook to get the current state of a job from the centralized events store.
 * This replaces the previous implementation that derived state from filtered events.
 */
export function useJobState({
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
  return useJobStateFromStore({ runId, jobId });
}
