import { useMemo } from "react";
import { deriveJobState, type JobState } from "../lib/job-state";
import { Event } from "@/bindings";
import { eq, useLiveQuery } from "@tanstack/react-db";
import { runEventsCollection } from "@/db/run-events";

export type RunStatus = "idle" | "running" | "completed" | "failed";

export type RunState = {
  /** Overall state of the run */
  state: RunStatus;
  /** Number of jobs currently running */
  jobsRunning: number;
  /** Number of jobs that completed successfully */
  jobsCompleted: number;
  /** Number of jobs that failed */
  jobsFailed: number;
  /** Number of jobs pending */
  jobsPending: number;
};

/**
 * Hook to get the overall status of a run.
 * Aggregates status across all jobs in the run.
 * Useful for showing run status in the sidebar.
 */
export function useRunState({ runId }: { runId: string }): {
  data: RunState | undefined;
  events: Event[];
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
        .orderBy(({ events }) => events.sequence_number, "desc")
        .limit(1000),
    [runId]
  );

  const data = useMemo<RunState | undefined>(() => {
    if (!events) return undefined;

    // Group events by job_id to get per-job status
    const eventsByJob = new Map<string, typeof events>();
    for (const event of events) {
      if (event.job_id) {
        const jobEvents = eventsByJob.get(event.job_id) ?? [];
        jobEvents.push(event);
        eventsByJob.set(event.job_id, jobEvents);
      }
    }

    // Derive status for each job and aggregate
    let jobsRunning = 0;
    let jobsCompleted = 0;
    let jobsFailed = 0;
    let jobsPending = 0;

    for (const jobEvents of eventsByJob.values()) {
      const jobState: JobState = deriveJobState(jobEvents);
      jobsRunning += jobState.runningCount;
      jobsCompleted += jobState.completedCount;
      jobsFailed += jobState.erroredCount;
      jobsPending += jobState.pendingCount;
    }

    // Determine overall state
    let state: RunStatus = "idle";
    if (jobsRunning > 0 || jobsPending > 0) {
      state = "running";
    } else if (jobsFailed > 0) {
      state = "failed";
    } else if (jobsCompleted > 0) {
      state = "completed";
    }

    return {
      state,
      jobsRunning,
      jobsCompleted,
      jobsFailed,
      jobsPending,
    };
  }, [events]);

  return {
    data,
    events,
    isLoading,
    isError,
  };
}
