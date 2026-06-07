import { useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import { createTransaction } from "@tanstack/react-db";
import { commands, LivestreamCursor } from "@/bindings";
import { workflowCollection } from "@/db/workflow";
import { runsCollection } from "@/db/runs";
import { runEventsCollection } from "@/db/run-events";

const DEFAULT_POLL_INTERVAL = 1000; // 1 second
const DEFAULT_WORKFLOWS_LIMIT = 50;
const DEFAULT_RUNS_LIMIT = 50;
const DEFAULT_EVENTS_LIMIT = 100;

interface SyncProviderProps {
  pollInterval?: number;
}

export function SyncProvider({
  pollInterval = DEFAULT_POLL_INTERVAL,
}: SyncProviderProps) {
  // Store cursor in ref to persist across polls without triggering re-renders
  const cursorRef = useRef<LivestreamCursor | null>(null);

  useQuery({
    queryKey: ["livestream-sync"],
    throwOnError: true,
    queryFn: async () => {
      const result = await commands.livestream({
        cursor: cursorRef.current ?? {
          workflows: { workflow_id: null, limit: DEFAULT_WORKFLOWS_LIMIT },
          runs: { run_id: null, limit: DEFAULT_RUNS_LIMIT },
          events: {
            event_id: null,
            sequence_number: null,
            limit: DEFAULT_EVENTS_LIMIT,
          },
        },
      });

      if (result.status !== "ok") throw new Error(result.error);

      const { workflows, runs, events, next_cursor } = result.data;

      // TODO: Write all data to collections in a single transaction
      if (workflows.length > 0) workflowCollection.insert(workflows);
      if (runs.length > 0) {
        console.log("runs insert", runs);
        runsCollection.insert(
          runs.map((run) => ({
            ...run,
            workflow_id: "019c579c-7790-72f0-9237-1c6ba5365d95", // run.workflow_version_id, // TODO: Fix
          }))
        );
      }
      if (events.length > 0) runEventsCollection.insert(events);

      // Update cursor for next poll
      cursorRef.current = next_cursor;

      return result.data;
    },
    refetchInterval: pollInterval,
    refetchIntervalInBackground: true,
  });

  return <></>;
}
