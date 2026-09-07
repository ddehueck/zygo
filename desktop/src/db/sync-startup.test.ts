import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SyncConfig } from "@tanstack/db";
import type { SyncCursor, SyncDelta, Tag } from "@/bindings";

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

let stream: ReturnType<typeof deferred<StreamResult>>;

// Drain the finite startup/loading promise chain without timing-dependent sleeps.
async function flushMicrotasks() {
  for (let i = 0; i < 10; i++) await Promise.resolve();
}

function channels(call = 0) {
  const args = mocks.openSyncChannel.mock.calls[call];
  expect(args).toHaveLength(2);
  return args as [{ onmessage: (delta: SyncDelta) => void }, { onmessage: (ready: null) => void }];
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
  stream = deferred<StreamResult>();
  mocks.openSyncChannel.mockReturnValue(stream.promise);
  mocks.loadSyncableData.mockResolvedValue(page());
});

describe("sync startup readiness", () => {
  it("shares the pending and resolved readiness promise, not the stream lifetime", async () => {
    const { syncClient } = await import("./sync-client");
    const first = syncClient.start();
    expect(first).toBeInstanceOf(Promise);
    expect(syncClient.start()).toBe(first);
    const settled = vi.fn();
    void first.then(settled);
    await flushMicrotasks();
    expect(settled).not.toHaveBeenCalled();
    expect(mocks.openSyncChannel).toHaveBeenCalledTimes(1);

    channels()[1].onmessage(null);
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

    channels()[1].onmessage(null);
    await flushMicrotasks();
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

    channels()[1].onmessage(null);
    await flushMicrotasks();
    expect(mocks.loadSyncableData).toHaveBeenCalledTimes(2);
    expect(first.markReady).toHaveBeenCalledOnce();
    expect(second.markReady).toHaveBeenCalledOnce();
    first.cleanup();
    second.cleanup();
  });

  it("buffers events before readiness and during pagination, then replays them in order", async () => {
    const lastPage = deferred<ReturnType<typeof page>>();
    const original = tag(2, "snapshot");
    const updated = tag(2, "live");
    const inserted = tag(3, "new");
    mocks.loadSyncableData
      .mockResolvedValueOnce(page([original], { id: 2 }))
      .mockReturnValueOnce(lastPage.promise);
    const target = await collection();
    const [delta, ready] = channels();
    delta.onmessage({ entity: "tag", change_id: 1, change: { operation: "update", row: updated } });
    expect(target.write).not.toHaveBeenCalled();
    ready.onmessage(null);
    await flushMicrotasks();
    expect(mocks.loadSyncableData).toHaveBeenNthCalledWith(2, {
      entity: "tag",
      cursor: { id: 2 },
      limit: 1000,
    });
    delta.onmessage({
      entity: "tag",
      change_id: 2,
      change: { operation: "insert", row: inserted },
    });
    delta.onmessage({ entity: "tag", change_id: 3, change: { operation: "delete", id: 2 } });
    expect(target.write.mock.calls).toEqual([[{ type: "insert", value: original }]]);
    expect(target.markReady).not.toHaveBeenCalled();

    lastPage.resolve(page());
    await flushMicrotasks();
    expect(target.write.mock.calls).toEqual([
      [{ type: "insert", value: original }],
      [{ type: "update", value: updated }],
      [{ type: "insert", value: inserted }],
      [{ type: "delete", key: 2 }],
    ]);
    expect(target.markReady).toHaveBeenCalledOnce();
    const commitOrder = target.commit.mock.invocationCallOrder;
    expect(commitOrder[commitOrder.length - 1]).toBeLessThan(
      target.markReady.mock.invocationCallOrder[0]!,
    );
    expect(target.markError).not.toHaveBeenCalled();
    target.cleanup();
  });

  it("does not load a collection cleaned up before readiness", async () => {
    const target = await collection();
    target.cleanup();
    channels()[1].onmessage(null);
    await flushMicrotasks();
    expect(mocks.loadSyncableData).not.toHaveBeenCalled();
    expect(target.write).not.toHaveBeenCalled();
    expect(target.markReady).not.toHaveBeenCalled();
    expect(target.markError).not.toHaveBeenCalled();
  });

  it.each(["rejection", "error result", "unexpected end"] as const)(
    "rejects startup and marks collections errored without loading on %s",
    async (failure) => {
      const target = await collection();
      const { syncClient } = await import("./sync-client");
      const onError = vi.fn();
      const unsubscribe = syncClient.subscribe("tag", vi.fn(), onError);
      const message =
        failure === "unexpected end" ? "Sync stream ended unexpectedly" : "startup failed";
      const rejection = expect(syncClient.start()).rejects.toThrow(message);
      if (failure === "rejection") stream.reject(new Error(message));
      else if (failure === "error result") stream.resolve({ status: "error", error: { message } });
      else stream.resolve({ status: "ok", data: null });
      await rejection;
      await flushMicrotasks();
      expect(mocks.loadSyncableData).not.toHaveBeenCalled();
      expect(target.markReady).not.toHaveBeenCalled();
      expect(target.markError).toHaveBeenCalledExactlyOnceWith(
        expect.objectContaining({ message }),
      );
      expect(onError).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({ message }));
      unsubscribe();
      target.cleanup();
    },
  );

  it("notifies on failure after readiness and starts a fresh readiness wait on retry", async () => {
    const { syncClient } = await import("./sync-client");
    const onError = vi.fn();
    const unsubscribe = syncClient.subscribe("tag", vi.fn(), onError);
    const first = syncClient.start();
    channels()[1].onmessage(null);
    await first;
    const error = new Error("stream disconnected");
    stream.reject(error);
    await flushMicrotasks();
    expect(onError).toHaveBeenCalledExactlyOnceWith(error);
    await expect(first).resolves.toBeUndefined();

    mocks.openSyncChannel.mockReturnValue(deferred<StreamResult>().promise);
    const retry = syncClient.start();
    expect(retry).not.toBe(first);
    expect(syncClient.start()).toBe(retry);
    expect(mocks.openSyncChannel).toHaveBeenCalledTimes(2);
    const settled = vi.fn();
    void retry.then(settled);
    await flushMicrotasks();
    expect(settled).not.toHaveBeenCalled();
    channels(1)[1].onmessage(null);
    await retry;
    expect(settled).toHaveBeenCalledOnce();
    unsubscribe();
  });
});
