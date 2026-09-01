import { DbClient } from "@tanstack/db";
import { QueryClient } from "@tanstack/query-core";

export const syncEntityRefreshOptions = {
  // CDC events are the live-update mechanism; refetching can apply an older
  // full snapshot after a newer CDC update.
  staleTime: Infinity,
  refetchOnWindowFocus: false,
  refetchOnReconnect: false,
};

export const queryClient = new QueryClient();
export const tdb = new DbClient({ queryClient });
