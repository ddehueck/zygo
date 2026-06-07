import type { Event } from "@/bindings";

export type JobStatus =
  | "idle"
  | "pending"
  | "running"
  | "completed"
  | "errored";

export type JobState = {
  /** Current aggregate status of the job */
  status: JobStatus;
  /** Number of job instances currently pending (requested but not started) */
  pendingCount: number;
  /** Number of job instances currently running */
  runningCount: number;
  /** Number of job instances that completed successfully */
  completedCount: number;
  /** Number of job instances that failed */
  erroredCount: number;
};

/**
 * Derives the current state of a job from its events.
 * Tracks each job execution through its lifecycle and aggregates counts.
 */
export function deriveJobState(events: Event[]): JobState {
  // Track state per execution (we don't have execution IDs, so we count transitions)
  let pendingCount = 0;
  let runningCount = 0;
  let completedCount = 0;
  let erroredCount = 0;

  // Sort events by sequence number to process in order
  const sortedEvents = [...events].sort((a, b) => {
    const seqA = a.sequence_number ?? 0n;
    const seqB = b.sequence_number ?? 0n;
    if (seqA < seqB) return -1;
    if (seqA > seqB) return 1;
    return 0;
  });

  for (const event of sortedEvents) {
    switch (event.event_kind) {
      case "job_requested":
        pendingCount++;
        break;
      case "job_started":
        // Move from pending to running
        if (pendingCount > 0) pendingCount--;
        runningCount++;
        break;
      case "job_succeeded":
        // Move from running to completed
        if (runningCount > 0) runningCount--;
        completedCount++;
        break;
      case "job_failed":
        // Move from running to errored
        if (runningCount > 0) runningCount--;
        erroredCount++;
        break;
    }
  }

  // Determine overall status based on counts
  let status: JobStatus = "idle";
  if (runningCount > 0) {
    status = "running";
  } else if (pendingCount > 0) {
    status = "pending";
  } else if (erroredCount > 0) {
    status = "errored";
  } else if (completedCount > 0) {
    status = "completed";
  }

  return {
    status,
    pendingCount,
    runningCount,
    completedCount,
    erroredCount,
  };
}
