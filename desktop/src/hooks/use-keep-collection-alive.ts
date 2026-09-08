import { useEffect } from "react";
import type { Collection } from "@tanstack/db";

export function useKeepCollectionAlive<T extends object, TKey extends string | number>(
  collection: Pick<Collection<T, TKey>, "subscribeChanges">,
  enabled = true,
) {
  useEffect(() => {
    if (!enabled) return;

    // Retain the collection without requiring a view to consume its changes.
    const subscription = collection.subscribeChanges(() => {});
    return () => subscription.unsubscribe();
  }, [collection, enabled]);
}
