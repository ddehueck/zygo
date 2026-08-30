import { listen } from "@tauri-apps/api/event";
import { useEffect, useEffectEvent, useRef } from "react";
import { commands, type SyncDelta, type WorkflowRunSummary } from "../bindings";
import { useWorkflowRunsActions, useWorkflowRunsCollection } from "../db/workflow-run-summaries";
import { assertNever } from "../utils";

type SyncPoke = Record<never, never>;

const SYNC_POKE_EVENT = "sync-poke";
const WORKFLOW_RUN_SUMMARY = "workflow_run_summary";
const MAX_DELTAS = 100;

/**
 * This component is to be added at the root level to sync in updates
 * from the rust backend that affect workflow run state.
 */
export function SyncWorkflowUpdates() {
  const collection = useWorkflowRunsCollection();
  const { upsert } = useWorkflowRunsActions();
  const changeId = useRef(0);
  const pendingSync = useRef(Promise.resolve());
  const initialSync = useRef<Promise<void> | null>(null);

  const applyPoke = useEffectEvent(async () => {
    // The initial snapshot and CDC updates share the same collection. Wait for
    // the snapshot to be applied before replaying CDC, otherwise an older
    // in-flight snapshot can overwrite a newer update that was just applied.
    await initialSync.current;

    const nextChangeId = await processPoke(changeId.current, upsert);
    if (nextChangeId !== null) {
      changeId.current = nextChangeId;
    }
  });

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    initialSync.current = collection.preload().catch((error: unknown) => {
      console.error("failed to preload workflow run summaries", error);
    });

    const queuePoke = () => {
      pendingSync.current = enqueueApplyPoke(pendingSync.current, applyPoke).catch(
        (error: unknown) => {
          console.error("failed to apply workflow sync updates", error);
        },
      );
    };

    // Reconcile once after startup as well as on events. This covers updates
    // emitted while the Tauri listener is being registered.
    queuePoke();

    void listen<SyncPoke>(SYNC_POKE_EVENT, () => {
      console.log("sync-poke event received");
      queuePoke();
    }).then((cleanup) => {
      if (disposed) {
        cleanup();
      } else {
        unlisten = cleanup;
      }
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [collection]);

  return null;
}

function enqueueApplyPoke(
  pendingSync: Promise<void>,
  applyPoke: () => Promise<void>,
): Promise<void> {
  return pendingSync.then(() => applyPoke());
}

async function processPoke(
  currentChangeId: number,
  upsert: (summary: WorkflowRunSummary) => void,
): Promise<number | null> {
  const result = await commands.getSyncDeltas({
    since: currentChangeId,
    max_deltas: MAX_DELTAS,
  });

  if (result.status === "error") {
    throw new Error(result.error);
  }

  for (const delta of result.data.deltas) {
    applyDelta(delta, upsert);
  }

  if (result.data.next_change_id === null) {
    return null;
  }

  const confirmResult = await commands.confirmSync({
    change_id: result.data.next_change_id,
  });
  if (confirmResult.status === "error") {
    throw new Error(confirmResult.error);
  }

  return result.data.next_change_id;
}

function applyDelta(delta: SyncDelta, upsert: (summary: WorkflowRunSummary) => void): void {
  switch (delta.operation) {
    case "resync":
      break;
    case "delete":
      break;
    case "upsert":
      switch (delta.entity) {
        case WORKFLOW_RUN_SUMMARY:
          upsert(delta.data as WorkflowRunSummary);
          break;
        case "workflow_run":
          break;
        default:
          assertNever(delta.entity);
      }
      break;
    default:
      assertNever(delta);
  }
}
