import { useMemo } from "react";
import { Event } from "@/bindings";
import { assertNever } from "@/lib/utils";
import { runEventsCollection } from "@/db/run-events";
import { eq, useLiveQuery } from "@tanstack/react-db";

type NodeFilter =
  | { type: "job"; jobId: string }
  | { type: "channel"; channelId: string };

export function useRunEvents({
  runId,
  node,
}: {
  runId: string;
  node: NodeFilter;
}) {
  const { data: events, ...rest } = useLiveQuery(
    (q) =>
      q
        .from({ events: runEventsCollection })
        .where(({ events }) => eq(events.workflow_run_id, runId))
        .orderBy(({ events }) => events.sequence_number, "desc")
        .limit(1000), // TODO: This should be a UI pagination limit
    [runId]
  );

  // ================================
  // FILTERING EVENTS
  // ================================

  const filteredEvents = useMemo(() => {
    if (!events) return undefined;

    let filtered: Event[];
    switch (node.type) {
      case "job":
        filtered = events.filter((event) => event.job_id === node.jobId);
        break;
      case "channel":
        filtered = events.filter(
          (event) => event.channel_id === node.channelId
        );
        break;
      default:
        assertNever(node);
    }

    // TODO: Sorting utils for this?
    // TODO: Timestamp type should be a date?
    return filtered.sort((a, b) => {
      if (!a.timestamp && !b.timestamp) return 0;
      if (!a.timestamp) return 1;
      if (!b.timestamp) return -1;
      return b.timestamp.localeCompare(a.timestamp);
    });
  }, [events, node]);

  return {
    ...rest,
    data: filteredEvents,
  };
}
