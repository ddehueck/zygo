import { createCollection, type CollectionConfig, type SyncConfig } from "@tanstack/db";
import { invoke } from "@tauri-apps/api/core";
import { commands, type SyncDelta, type SyncEntityKind, type SyncUpsert } from "@/bindings";
import { syncClient } from "./sync";

type Entity = {
  id: number;
};

type Snapshot<T> = {
  rows: T[];
  cursor: number;
};

/**
 * For building a custom collection that syncs with the backend using the custom sync protocol
 * https://tanstack.com/db/latest/docs/guides/collection-options-creator#when-to-create-a-custom-collection
 */
export function syncCollectionOptions<T extends Entity>(
  table: SyncEntityKind,
): CollectionConfig<T> {
  return {
    id: table,
    getKey: (row) => row.id,
    syncMode: "eager",
    sync: initSync<T>(table),
  };
}

function initSync<T extends Entity>(table: SyncEntityKind): SyncConfig<T> {
  return {
    sync: ({ begin, write, commit, markReady, markError }) => {
      let disposed = false;
      let unsubscribe: (() => void) | undefined;

      async function initialize() {
        try {
          await syncClient.start();
          if (disposed) return;

          unsubscribe = syncClient.subscribe(table, (delta) => {
            if (disposed) return;

            begin();
            writeDelta(delta, write);
            commit();
          });

          // TODO: Implement a get snapshot for each sync entity
          // const snapshot = await syncClient.snapshot<T>(table);
          const snapshot = { rows: [], cursor: 0 } as Snapshot<T>;
          if (disposed) return;

          begin();

          for (const row of snapshot.rows) {
            write({
              type: "insert",
              value: row,
            });
          }

          commit();

          markReady();
        } catch (error) {
          if (!disposed) markError(error);
        }
      }

      void initialize();

      return () => {
        disposed = true;
        unsubscribe?.();
      };
    },
  };
}

// todo: rewrite sync delta to be in sync with the snapshot loading
// const pending = buffered.filter(
//   (delta) => delta.seq > snapshot.cursor,
// )
// for (const delta of pending) {
//  writeDelta(delta, write);
//}
function writeDelta<T>(delta: SyncDelta, write: (change: any) => void) {
  switch (delta.operation) {
    case "upsert":
      write({
        type: "insert",
        value: delta.value,
      });
      break;

    case "update":
      write({
        type: "update",
        value: delta.value,
      });
      break;

    case "delete":
      write({
        type: "delete",
        key: delta.key,
      });
      break;
  }
}
