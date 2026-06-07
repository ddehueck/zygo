import type { Event } from "@/bindings";

// ================================
// TYPES
// ================================

export type JobExecutionStatus =
  | "pending"
  | "running"
  | "completed"
  | "errored";

export type JobExecution = {
  idempotencyKey: string;
  status: JobExecutionStatus;
  events: Event[];
  startedAt: string | null;
  endedAt: string | null;
};

export type JobState = {
  status: "idle" | "pending" | "running" | "completed" | "errored";
  pendingCount: number;
  runningCount: number;
  completedCount: number;
  erroredCount: number;
  executions: JobExecution[];
};

export type ChannelState = {
  itemCount: number;
  events: Event[];
};

// ================================
// UTILITY FUNCTIONS
// ================================

/**
 * Sort events by sequence number in ascending order
 */
function sortEventsBySequence(events: Event[]): Event[] {
  return [...events].sort((a, b) => {
    const seqA = a.sequence_number ?? 0n;
    const seqB = b.sequence_number ?? 0n;
    if (seqA < seqB) return -1;
    if (seqA > seqB) return 1;
    return 0;
  });
}

/**
 * Derive the status of a job execution from its events
 */
function deriveExecutionStatus(events: Event[]): JobExecutionStatus {
  // Check events in reverse order (most recent first) to find terminal state
  for (let i = events.length - 1; i >= 0; i--) {
    const event = events[i];
    if (event.event_kind === "job_succeeded") return "completed";
    if (event.event_kind === "job_failed") return "errored";
    if (event.event_kind === "job_started") return "running";
  }
  return "pending";
}

// ================================
// RUN EVENTS STORE CLASS
// ================================

export class RunEventsStore {
  private events: Event[] = [];
  private eventsByJobId: Map<string, Event[]> = new Map();
  private eventsByChannelId: Map<string, Event[]> = new Map();
  private jobExecutionsByJobId: Map<string, Map<string, Event[]>> = new Map();

  constructor(events: Event[]) {
    this.events = sortEventsBySequence(events);
    this.indexEvents();
  }

  /**
   * Index events for efficient querying
   */
  private indexEvents(): void {
    for (const event of this.events) {
      // Index by job_id
      if (event.job_id) {
        const jobEvents = this.eventsByJobId.get(event.job_id) ?? [];
        jobEvents.push(event);
        this.eventsByJobId.set(event.job_id, jobEvents);

        // Index by idempotency key for job executions
        if (event.job_run_id) {
          let jobExecutions = this.jobExecutionsByJobId.get(event.job_id);
          if (!jobExecutions) {
            jobExecutions = new Map();
            this.jobExecutionsByJobId.set(event.job_id, jobExecutions);
          }

          const executionEvents =
            jobExecutions.get(event.job_run_id) ?? [];
          executionEvents.push(event);
          jobExecutions.set(event.job_run_id, executionEvents);
        }
      }

      // Index by channel_id
      if (event.channel_id) {
        const channelEvents =
          this.eventsByChannelId.get(event.channel_id) ?? [];
        channelEvents.push(event);
        this.eventsByChannelId.set(event.channel_id, channelEvents);
      }
    }
  }

  getAllEvents(): Event[] {
    return this.events;
  }

  getEventsByJobId(jobId: string): Event[] {
    return this.eventsByJobId.get(jobId) ?? [];
  }

  getEventsByChannelId(channelId: string): Event[] {
    return this.eventsByChannelId.get(channelId) ?? [];
  }

  getJobExecutions(jobId: string): JobExecution[] {
    const executionsMap = this.jobExecutionsByJobId.get(jobId);
    if (!executionsMap) return [];

    const executions: JobExecution[] = [];

    for (const [idempotencyKey, events] of executionsMap) {
      const sortedEvents = sortEventsBySequence(events);
      const status = deriveExecutionStatus(sortedEvents);

      // Find started and ended timestamps
      const startedEvent = sortedEvents.find(
        (e) => e.event_kind === "job_started"
      );
      const endedEvent = sortedEvents.find(
        (e) => e.event_kind === "job_succeeded" || e.event_kind === "job_failed"
      );

      executions.push({
        idempotencyKey,
        status,
        events: sortedEvents,
        startedAt: startedEvent?.timestamp ?? null,
        endedAt: endedEvent?.timestamp ?? null,
      });
    }

    // Sort executions by start time (most recent first)
    return executions.sort((a, b) => {
      if (!a.startedAt && !b.startedAt) return 0;
      if (!a.startedAt) return 1;
      if (!b.startedAt) return -1;
      return b.startedAt.localeCompare(a.startedAt);
    });
  }

  /**
   * Get the aggregate state for a job
   */
  getJobState(jobId: string): JobState {
    const events = this.getEventsByJobId(jobId);
    const executions = this.getJobExecutions(jobId);

    // Count pending from job_requested events without idempotency keys
    // (these are requests that haven't started yet)
    let pendingCount = 0;
    let runningCount = 0;
    let completedCount = 0;
    let erroredCount = 0;

    // Count from executions that have idempotency keys
    for (const execution of executions) {
      switch (execution.status) {
        case "pending":
          pendingCount++;
          break;
        case "running":
          runningCount++;
          break;
        case "completed":
          completedCount++;
          break;
        case "errored":
          erroredCount++;
          break;
      }
    }

    // Count job_requested events that don't have a corresponding execution yet
    const requestedEvents = events.filter(
      (e) => e.event_kind === "job_requested"
    );
    const executionCount = executions.length;
    const unresolvedRequests = requestedEvents.length - executionCount;
    if (unresolvedRequests > 0) {
      pendingCount += unresolvedRequests;
    }

    // Determine overall status
    let status: JobState["status"] = "idle";
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
      executions,
    };
  }

  getChannelState(channelId: string): ChannelState {
    const events = this.getEventsByChannelId(channelId);
    const itemCount = events.filter(
      (e) => e.event_kind === "channel_item_inserted"
    ).length;

    return {
      itemCount,
      events: sortEventsBySequence(events),
    };
  }
}

/**
 * Create a new RunEventsStore from an array of events
 */
export function createRunEventsStore(events: Event[]): RunEventsStore {
  return new RunEventsStore(events);
}
