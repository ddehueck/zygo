import { Channel } from "@tauri-apps/api/core";
import { commands, type SyncDelta, type SyncEntityKind, type SyncUpsert } from "@/bindings";
import { deleteCollectionItem, upsertCollectionItem } from "@/db/collection-helpers";
import {
  dataReferencesCollection,
  jobRunsCollection,
  tagsCollection,
  workflowRunsCollection,
} from "@/db/collections";
import { assertNever } from "@/utils";

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

type Listener = (change: SyncDelta) => void;

/**
 * Custom sync client to build a custom sync protocol for use in a custom TanstackDB Collection
 */
class SyncClient {
  private channel = new Channel<SyncDelta>();
  private listeners = new Map<SyncEntityKind, Set<Listener>>();
  private startPromise?: Promise<void>;

  constructor() {
    this.channel.onmessage = (delta) => {
      const entity = getEntity(delta);
      if (!entity) return; // TODO: Remove resync

      const listeners = this.listeners.get(entity);
      if (!listeners) return;

      for (const listener of listeners) {
        listener(delta);
      }
    };
  }

  start(): Promise<void> {
    if (!this.startPromise) {
      this.startPromise = commands.sync(this.channel).then((r) => {
        if (r.status === "error")
          console.error("sync command failed", r.error);
        console.log("sync initiated completed", r);
      }).catch((e) => console.error((e)));
    }

    return this.startPromise;
  }

  subscribe(table: SyncEntityKind, listener: Listener): () => void {
    let listeners = this.listeners.get(table);

    if (!listeners) {
      listeners = new Set();
      this.listeners.set(table, listeners);
    }

    listeners.add(listener);

    return () => {
      listeners!.delete(listener);

      if (listeners!.size === 0) {
        this.listeners.delete(table);
      }
    };
  }
}

export const syncClient = new SyncClient();

function getEntity(delta: SyncDelta): SyncEntityKind | null {
  switch (delta.operation) {
    case "resync":
      return null;
    case "delete":
      return delta.entity;
    case "upsert":
      return delta.payload.entity;
    default:
      assertNever(delta);
  }
