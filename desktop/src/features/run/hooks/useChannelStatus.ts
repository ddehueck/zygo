import { useChannelStateFromStore, type ChannelState } from "./useRunEventsStore";

export type { ChannelState as ChannelStatus };

/**
 * Hook to get the current state of a channel from the centralized events store.
 * This replaces the previous implementation that derived state from filtered events.
 */
export function useChannelStatus({
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
  return useChannelStateFromStore({ runId, channelId });
}
