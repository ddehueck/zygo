import { cn } from "@/lib/utils";
import { useChannelStateFromStore } from "../hooks/useRunEventsStore";
import type { Event } from "@/bindings";
import { Blade, BladeHeader, BladeContent } from "@/components/layout/blade";

type ChannelEventsSidebarProps = {
  runId: string;
  channelId: string;
  channelName: string;
  className?: string;
};

function formatTimestamp(timestamp: string | null): string {
  if (!timestamp) return "—";
  try {
    const date = new Date(timestamp);
    return date.toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  } catch {
    return "—";
  }
}

function ChannelEventRow({ event }: { event: Event }) {
  return (
    <div className="flex items-center gap-3 rounded-md border border-border/50 bg-card/50 px-3 py-2 text-sm">
      <div className="flex size-5 items-center justify-center rounded-full bg-chart-2/20 text-chart-2">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="currentColor"
          className="size-3"
        >
          <path
            fillRule="evenodd"
            d="M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16Zm.75-11.25a.75.75 0 0 0-1.5 0v4.59L7.3 9.24a.75.75 0 0 0-1.1 1.02l3.25 3.5a.75.75 0 0 0 1.1 0l3.25-3.5a.75.75 0 1 0-1.1-1.02l-1.95 2.1V6.75Z"
            clipRule="evenodd"
          />
        </svg>
      </div>
      <div className="min-w-0 flex-1">
        <p className="text-xs text-foreground">Item inserted</p>
        <p className="mt-0.5 text-xs text-muted-foreground">
          {formatTimestamp(event.timestamp)}
        </p>
      </div>
    </div>
  );
}

export function ChannelEventsSidebar({
  runId,
  channelId,
  channelName,
  className,
}: ChannelEventsSidebarProps) {
  const { data: channelState, isLoading } = useChannelStateFromStore({
    runId,
    channelId,
  });

  return (
    <Blade position="right" className={cn(className)}>
      <BladeHeader>
        <div className="flex items-center gap-2">
          <div className="flex size-6 items-center justify-center rounded bg-chart-2/10 text-chart-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 20 20"
              fill="currentColor"
              className="size-3.5"
            >
              <path d="M12.5 2.75a.75.75 0 0 0-1.5 0v14.5a.75.75 0 0 0 1.5 0V2.75ZM8.5 5.5a.75.75 0 0 0-1.5 0v9a.75.75 0 0 0 1.5 0v-9ZM16.5 5.5a.75.75 0 0 0-1.5 0v9a.75.75 0 0 0 1.5 0v-9ZM4.5 8.25a.75.75 0 0 0-1.5 0v3.5a.75.75 0 0 0 1.5 0v-3.5Z" />
            </svg>
          </div>
          <div className="min-w-0 flex-1">
            <h3 className="truncate text-sm font-medium text-sidebar-foreground">
              {channelName}
            </h3>
            <p className="text-xs text-muted-foreground">Channel Events</p>
          </div>
        </div>
      </BladeHeader>

      {/* Stats Bar */}
      {channelState && channelState.itemCount > 0 && (
        <div className="shrink-0 border-b px-4 py-2">
          <div className="flex items-center gap-1 text-xs text-chart-2">
            <span className="font-medium">{channelState.itemCount}</span>
            <span className="text-muted-foreground">
              item{channelState.itemCount !== 1 ? "s" : ""} inserted
            </span>
          </div>
        </div>
      )}

      <BladeContent>
        {isLoading ? (
          <div className="py-8 text-center text-sm text-muted-foreground">
            Loading...
          </div>
        ) : !channelState || channelState.events.length === 0 ? (
          <div className="py-8 text-center">
            <p className="text-sm text-muted-foreground">No events yet</p>
            <p className="mt-1 text-xs text-muted-foreground/70">
              Events will appear here when items are inserted
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {channelState.events
              .filter((e) => e.event_kind === "channel_item_inserted")
              .map((event) => (
                <ChannelEventRow key={event.id} event={event} />
              ))}
          </div>
        )}
      </BladeContent>
    </Blade>
  );
}
