import { Channel } from "@tauri-apps/api/core";
import { commands, type SyncDelta, type SyncEntityKind, type SyncUpsert } from "../../bindings";
import { workflowRuns } from "../../db/workflow-runs";
import { jobRuns } from "../../db/job-runs";
import { assertNever } from "../../utils";

type CollectionsByEntity = {
  workflow_run: typeof workflowRuns;
  job_run: typeof jobRuns;
};

const collections: CollectionsByEntity = {
  workflow_run: workflowRuns,
  job_run: jobRuns,
};

const SyncChannel = new Channel<SyncDelta>();

SyncChannel.onmessage = (message) => {
  console.log("got sync event", message);
  switch (message.operation) {
    case "resync":
      // TODO
      break;
    case "delete":
      applyDelete(message.entity, message.id);
      break;
    case "upsert":
      applyUpsert(message.payload);
      break;
    default:
      assertNever(message);
  }
};

function applyDelete(entity: SyncEntityKind, id: string) {
  collections[entity].utils.writeDelete(id);
}

type SyncUpsertFor<K extends SyncEntityKind> = Extract<SyncUpsert, { entity: K }>;

function applyUpsert<K extends SyncEntityKind>(payload: SyncUpsertFor<K>) {
  collections[payload.entity].utils.writeUpsert(payload.data);
}

export async function startSync() {
  try {
    // Manual writes require each collection's sync context to be initialized.
    // Start the eager collections before connecting to CDC so the first delta
    // cannot arrive while a collection is still in its idle/loading state.
    await Promise.all(Object.values(collections).map((collection) => collection.preload()));
  } catch (error) {
    console.error("sync initialization failed", error);
    return;
  }

  try {
    const result = await commands.sync(SyncChannel);
    if (result.status === "error") {
      console.error("sync command failed", result.error);
    }
  } catch (error) {
    console.error("sync connection failed", error);
  }
}
