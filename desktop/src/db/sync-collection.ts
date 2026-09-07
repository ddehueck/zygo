import type { CollectionConfig, SyncConfig } from "@tanstack/db";
import {
  commands,
  type RowChange,
  type SyncCursor,
  type SyncEntityKind,
  type SyncPage,
} from "@/bindings";
import { syncClient, type SyncRow } from "./sync-client";

const PAGE_SIZE = 1000;

async function loadPage<E extends SyncEntityKind>(
  entity: E,
  cursor: SyncCursor | null,
): Promise<SyncPage<SyncRow<E>>> {
  const result = await commands.loadSyncableData({ entity, cursor, limit: PAGE_SIZE });
  if (result.status === "error") {
    throw Object.assign(new Error(result.error.message), { cause: result.error });
  }
  if (result.data.entity !== entity) {
    throw new Error(`Expected ${entity} page, received ${result.data.entity}`);
  }

  // Specta exports a union; the checked discriminator ties this page to E.
  return result.data.page as SyncPage<SyncRow<E>>;
}

/** Creates a full-table collection with its row type inferred from the sync entity. */
export function syncCollectionOptions<E extends SyncEntityKind>(
  entity: E,
): CollectionConfig<SyncRow<E>, number> {
  return {
    id: entity,
    getKey: (row) => row.id,
    syncMode: "eager", // right now we intentionally just load everything for simplicity.
    sync: initSync(entity),
  };
}

function initSync<E extends SyncEntityKind>(entity: E): SyncConfig<SyncRow<E>, number> {
  return {
    sync: ({ begin, write, commit, markReady, markError }) => {
      let disposed = false;
      let loading = true;
      let phase = "waiting for sync stream readiness";
      let unsubscribe: (() => void) | undefined;

      const keys = new Set<number>();
      const pending: RowChange<SyncRow<E>>[] = [];

      function fail(error: unknown, failedPhase = phase) {
        if (disposed) return;
        const message = error instanceof Error ? error.message : String(error);
        console.error(`[sync:${entity}] Failed while ${failedPhase}: ${message}`, error);
        disposed = true;
        unsubscribe?.();
        pending.length = 0;
        markError(error);
      }

      function writeChange(change: RowChange<SyncRow<E>>) {
        switch (change.operation) {
          case "insert":
          case "update":
            // Full-row CDC events can overlap with rows loaded by pagination.
            // TODO: Update snapshot to return a watermark so we can filter out stale events instead of blindly writing them.
            write({
              type: keys.has(change.row.id) ? "update" : "insert",
              value: change.row,
            });
            keys.add(change.row.id);
            break;
          case "delete":
            if (keys.delete(change.id)) write({ type: "delete", key: change.id });
            break;
        }
      }

      unsubscribe = syncClient.subscribe(
        entity,
        (change) => {
          if (disposed) return;
          if (loading) {
            pending.push(change);
            return;
          }
          try {
            begin();
            console.debug(`[sync:${entity}] Applying live change:`, change);
            writeChange(change);
            commit();
          } catch (error) {
            fail(error);
          }
        },
        (error) => fail(error, "receiving sync stream item of"),
      );

      async function initialize() {
        try {
          await syncClient.start();
          if (disposed) return;
          let cursor: SyncCursor | null = null;
          do {
            phase = `loading page (cursor: ${cursor?.id ?? "initial"})`;
            const page: SyncPage<SyncRow<E>> = await loadPage(entity, cursor);
            if (disposed) return;
            if (page.next && cursor && page.next.id >= cursor.id) {
              throw new Error(`Pagination did not advance for ${entity}`);
            }
            phase = `writing page (${page.data.length} rows)`;
            begin();
            for (const row of page.data) writeChange({ operation: "insert", row });
            commit();
            cursor = page.next;
          } while (cursor !== null);

          // This is not a CDC-consistent snapshot yet: the backend must supply
          // a snapshot watermark before stale buffered events can be filtered.
          phase = `replaying ${pending.length} buffered changes`;
          begin();
          for (const change of pending) writeChange(change);
          commit();
          pending.length = 0;
          loading = false;
          phase = "marking collection ready";
          markReady();
          phase = "applying live changes";
        } catch (error) {
          fail(error);
        }
      }

      void initialize();
      return () => {
        disposed = true;
        unsubscribe?.();
        pending.length = 0;
      };
    },
  };
}
