import type { Event } from "@/bindings";

export type ChannelState = {
  /** Number of items inserted into the channel */
  itemCount: number;
};

/**
 * Derives the current state of a channel from its events.
 * Counts the number of items that have been inserted.
 */
export function deriveChannelState(events: Event[]): ChannelState {
  const itemCount = events.filter(
    (e) => e.event_kind === "channel_item_inserted"
  ).length;

  return { itemCount };
}
