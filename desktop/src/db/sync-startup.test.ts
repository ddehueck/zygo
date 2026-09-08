import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { SyncConfig } from "@tanstack/db";
import type { SyncCursor, SyncDelta, Tag } from "@/bindings";
import { last } from "@/lib/arrays";

const mocks = vi.hoisted(() => ({
  openSyncChannel: vi.fn(),
  loadSyncableData: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  Channel: class<T> {
    onmessage: (message: T) => void = () => {};
  },
}));
vi.mock("@/bindings", () => ({ commands: mocks }));

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

type StreamResult = { status: "ok"; data: null } | { status: "error"; error: { message: string } };

let syncStream: ReturnType<typeof deferred<StreamResult>>;

// Drain the finite startup/loading promise chain without sleeping for some time period.
async function flushMicrotasks() {
  for (let i = 0; i < 10; i++) await Promise.resolve();
}

// Return the arguments of the nth call to openSyncChannel, asserting that it was called with the expected arguments.
function openSyncChannelArgs(call = 0) {
  const args = mocks.openSyncChannel.mock.calls[call];
  expect(args).toHaveLength(2);
  return {
    syncChannel: args[0] as { onmessage: (delta: SyncDelta) => void },
    readyChannel: args[1] as { onmessage: (ready: null) => void },
  };
}

function tag(id: number, value: string): Tag {
  return {
    id,
    value,
    workflow_run_id: 1,
    job_run_id: null,
    data_reference_id: null,
    created_at: "2026-01-01T00:00:00Z",
  };
}

function page(data: Tag[] = [], next: SyncCursor | null = null) {
  return { status: "ok" as const, data: { entity: "tag" as const, page: { data, next } } };
}

async function collection() {
  const { syncCollectionOptions } = await import("./sync-collection");
  const callbacks = {
    begin: vi.fn(),
    write: vi.fn(),
    commit: vi.fn(),
    markReady: vi.fn(),
    markError: vi.fn(),
  };
  // The adapter uses only these callbacks; no TanStack collection instance is needed.
  const cleanup = syncCollectionOptions("tag").sync!.sync(
    callbacks as unknown as Parameters<SyncConfig<Tag, number>["sync"]>[0],
  );
  expect(cleanup).toBeTypeOf("function");
  return { ...callbacks, cleanup: cleanup as () => void };
}

beforeEach(() => {
  vi.resetModules();
  vi.resetAllMocks();
  vi.spyOn(console, "error").mockImplementation(() => {});
  syncStream = deferred<StreamResult>();
  mocks.openSyncChannel.mockReturnValue(syncStream.promise);
  mocks.loadSyncableData.mockResolvedValue(page());
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("sync startup readiness", () => {
  it("shares the pending and resolved readiness promise, not the stream lifetime", async () => {
    const { syncClient } = await import("./sync-client");

    const first = syncClient.start();

    // Check singleton behavior of the startup promise.
    expect(first).toBeInstanceOf(Promise);
    expect(syncClient.start()).toBe(first);

    const settled = vi.fn();
    void first.then(settled);

    await flushMicrotasks();
    expect(settled).not.toHaveBeenCalled();
    // .start() should open a sync channel only once, even with multiple calls
    expect(mocks.openSyncChannel).toHaveBeenCalledTimes(1);

    // Mock recieving a ready channel message
    openSyncChannelArgs().readyChannel.onmessage(null);
    await first;
    expect(settled).toHaveBeenCalledOnce();
    expect(syncClient.start()).toBe(first);
    expect(mocks.openSyncChannel).toHaveBeenCalledTimes(1);
  });

  it("does not load or mark ready before the ready message", async () => {
    const target = await collection();
    await flushMicrotasks();
    expect(mocks.loadSyncableData).not.toHaveBeenCalled();
    expect(target.markReady).not.toHaveBeenCalled();
    expect(target.begin).not.toHaveBeenCalled();

    // Mock recieving a ready channel message
    openSyncChannelArgs().readyChannel.onmessage(null);
    await flushMicrotasks();

    // The collection should now be ready and have loaded the first page of data.
    expect(mocks.loadSyncableData).toHaveBeenCalledExactlyOnceWith({
      entity: "tag",
      cursor: null,
      limit: 1000,
    });
    expect(target.markReady).toHaveBeenCalledOnce();
    expect(target.markError).not.toHaveBeenCalled();
    target.cleanup();
  });

  it("lets concurrent collections wait on one startup command", async () => {
    const [first, second] = await Promise.all([collection(), collection()]);

    await flushMicrotasks();
    expect(mocks.openSyncChannel).toHaveBeenCalledTimes(1);
    expect(mocks.loadSyncableData).not.toHaveBeenCalled();
    expect(first.markReady).not.toHaveBeenCalled();
    expect(second.markReady).not.toHaveBeenCalled();

    // Mock recieving a ready channel message
    openSyncChannelArgs().readyChannel.onmessage(null);
    await flushMicrotasks();
    expect(mocks.loadSyncableData).toHaveBeenCalledTimes(2);
    expect(first.markReady).toHaveBeenCalledOnce();
    expect(second.markReady).toHaveBeenCalledOnce();

    first.cleanup();
    second.cleanup();
  });

  it("buffers events before ready message and during pagination, then replays them in order", async () => {
    const lastPage = deferred<ReturnType<typeof page>>();
    const original = tag(2, "snapshot");
    const updated = tag(2, "live");
    const inserted = tag(3, "new");

    mocks.loadSyncableData
      .mockResolvedValueOnce(page([original], { id: 2 }))
      .mockReturnValueOnce(lastPage.promise);

    const tagCollection = await collection();
    const { syncChannel, readyChannel } = openSyncChannelArgs();

    // Sync channel recieves a message before ready message has been recieved
    syncChannel.onmessage({
      entity: "tag",
      change_id: 1,
      change: { operation: "update", row: updated },
    });
    expect(tagCollection.write).not.toHaveBeenCalled();

    readyChannel.onmessage(null);
    await flushMicrotasks();

    // Data has loaded to the second page and we recieve deltas concurrently
    expect(mocks.loadSyncableData).toHaveBeenNthCalledWith(2, {
      entity: "tag",
      cursor: { id: 2 },
      limit: 1000,
    });
    syncChannel.onmessage({
      entity: "tag",
      change_id: 2,
      change: { operation: "insert", row: inserted },
    });
    syncChannel.onmessage({ entity: "tag", change_id: 3, change: { operation: "delete", id: 2 } });

    expect(tagCollection.write.mock.calls).toEqual([[{ type: "insert", value: original }]]);
    expect(tagCollection.markReady).not.toHaveBeenCalled();

    // Now we're done with loading the data snapshot
    lastPage.resolve(page());
    await flushMicrotasks();

    // Enforce order that we ended up writing to the collection
    expect(tagCollection.write.mock.calls).toEqual([
      [{ type: "insert", value: original }],
      [{ type: "update", value: updated }],
      [{ type: "insert", value: inserted }],
      [{ type: "delete", key: 2 }],
    ]);

    expect(tagCollection.markReady).toHaveBeenCalledOnce();
    expect(tagCollection.markError).not.toHaveBeenCalled();

    // Check that all commits were called befor markRead was.
    const commitOrder = tagCollection.commit.mock.invocationCallOrder;
    expect(last(commitOrder)).toBeLessThan(tagCollection.markReady.mock.invocationCallOrder[0]!);

    tagCollection.cleanup();
  });

  it("aborts loading data when an error occurs before ready message is recieved", async () => {
    const mockCollection = await collection();
    mockCollection.cleanup();

    openSyncChannelArgs().readyChannel.onmessage(null);
    await flushMicrotasks();

    expect(mocks.loadSyncableData).not.toHaveBeenCalled();
    expect(mockCollection.write).not.toHaveBeenCalled();
    expect(mockCollection.markReady).not.toHaveBeenCalled();
    expect(mockCollection.markError).not.toHaveBeenCalled();
  });

  it.each(["rejection", "error result", "unexpected end"] as const)(
    "rejects startup and marks collections errored without loading on %s",
    async (failure) => {
      const { SyncError, syncClient } = await import("./sync-client");

      const mockCollection = await collection();

      const onError = vi.fn();
      const unsubscribe = syncClient.subscribe("tag", vi.fn(), onError);
      const startupError = syncClient.start().catch((error: unknown) => error);

      switch (failure) {
        case "rejection":
          syncStream.reject(new Error("startup failed"));
          break;
        case "error result":
          syncStream.resolve({ status: "error", error: { message: "startup failed" } });
          break;
        case "unexpected end":
          syncStream.resolve({ status: "ok", data: null });
          break;
      }

      const error = await startupError;
      await flushMicrotasks();

      expect(error).toBeInstanceOf(SyncError);
      expect(error).toHaveProperty(
        "code",
        failure === "unexpected end" ? "stream_ended" : "stream_failed",
      );
      expect(mocks.loadSyncableData).not.toHaveBeenCalled();
      expect(mockCollection.markReady).not.toHaveBeenCalled();
      expect(mockCollection.markError).toHaveBeenCalledExactlyOnceWith(error);
      expect(onError).toHaveBeenCalledExactlyOnceWith(error);

      unsubscribe();
      mockCollection.cleanup();
    },
  );

  it("retries a failed open sync channel call properly", async () => {
    const { SyncError, syncClient } = await import("./sync-client");

    const onError = vi.fn();
    const unsubscribe = syncClient.subscribe("tag", vi.fn(), onError);
    const first = syncClient.start();

    // Receive ready message
    openSyncChannelArgs().readyChannel.onmessage(null);
    await first;

    // But then the sync stream breaks
    syncStream.reject(new Error("stream disconnected"));
    await flushMicrotasks();
    expect(onError).toHaveBeenCalledExactlyOnceWith(
      expect.objectContaining({ code: "stream_failed" }),
    );
    expect(onError.mock.calls[0]![0]).toBeInstanceOf(SyncError);
    await expect(first).resolves.toBeUndefined();

    // Set another return value in mock
    mocks.openSyncChannel.mockReturnValue(deferred<StreamResult>().promise);

    // Retry should start a new sync channel and return a new promise
    const retry = syncClient.start();
    expect(retry).not.toBe(first);
    expect(syncClient.start()).toBe(retry);
    expect(mocks.openSyncChannel).toHaveBeenCalledTimes(2);

    const settled = vi.fn();
    void retry.then(settled);

    await flushMicrotasks();
    expect(settled).not.toHaveBeenCalled();

    // Retry succeeds
    openSyncChannelArgs(1).readyChannel.onmessage(null);
    await retry;
    expect(settled).toHaveBeenCalledOnce();

    unsubscribe();
  });
});
