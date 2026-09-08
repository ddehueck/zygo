import { Channel } from "@tauri-apps/api/core";
import { commands, type RowChange, type SyncDelta, type SyncEntityKind } from "@/bindings";
import { errMsg } from "@/utils";
import { SyncError } from "./sync-error";

export { SyncError } from "./sync-error";

type RowsByEntity = {
  [D in SyncDelta as D["entity"]]: Extract<D["change"], { operation: "insert" }>["row"];
};

export type SyncRow<E extends SyncEntityKind> = RowsByEntity[E];

type Listener = (delta: SyncDelta) => void;

function asSyncError(error: unknown): SyncError {
  if (error instanceof SyncError) return error;
  return new SyncError(errMsg(error), "stream_failed");
}

class SyncClient {
  private channel = new Channel<SyncDelta>();
  private listeners = new Map<SyncEntityKind, Set<Listener>>();
  private errorListeners = new Set<(error: unknown) => void>();
  private ready: Promise<void> | undefined;

  constructor() {
    this.channel.onmessage = (delta) => {
      for (const listener of this.listeners.get(delta.entity) ?? []) {
        listener(delta);
      }
    };
  }

  start(): Promise<void> {
    if (this.ready) return this.ready;

    this.ready = new Promise<void>((resolve, reject) => {
      // The openSyncChannel command lives as long as the stream as it opens the sync channel.
      // We need an explicit ACK to guarentee its CDC cursor is set before we
      // start hydrating all the data. Hence, the onReady channel.

      const onReady = new Channel<null>();
      onReady.onmessage = () => resolve();

      void commands
        .openSyncChannel(this.channel, onReady)
        .then((result) => {
          if (result.status === "error") {
            throw new SyncError(result.error.message, "stream_failed");
          }
          throw new SyncError("Sync stream ended unexpectedly", "stream_ended");
        })
        .catch((error: unknown) => {
          const syncError = asSyncError(error);
          this.ready = undefined;
          reject(syncError);
          for (const listener of this.errorListeners) listener(syncError);
        });
    });
    return this.ready;
  }

  subscribe<E extends SyncEntityKind>(
    entity: E,
    listener: (change: RowChange<SyncRow<E>>) => void,
    onError: (error: unknown) => void,
  ): () => void {
    const wrapped: Listener = (delta) => {
      if (delta.entity !== entity) return;
      // We rely on the entity check above to ensure the type is correct, so we can safely cast here.
      listener(delta.change as RowChange<SyncRow<E>>);
    };

    let listeners = this.listeners.get(entity);
    if (!listeners) {
      listeners = new Set();
      this.listeners.set(entity, listeners);
    }

    listeners.add(wrapped);
    this.errorListeners.add(onError);

    return () => {
      listeners.delete(wrapped);
      this.errorListeners.delete(onError);
      if (listeners.size === 0) this.listeners.delete(entity);
    };
  }
}

export const syncClient = new SyncClient();
