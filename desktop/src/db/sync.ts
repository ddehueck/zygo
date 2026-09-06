import { Channel } from "@tauri-apps/api/core";
import { commands, type SyncDelta, type SyncEntityKind, type SyncUpsert } from "@/bindings";
import { deleteCollectionItem, upsertCollectionItem } from "@/db/collection-helpers";
import { jobRunsCollection, tagsCollection, workflowRunsCollection } from "@/db/collections";
import { assertNever } from "@/utils";

type CollectionsByEntity = {
  workflow_run: typeof workflowRunsCollection;
  job_run: typeof jobRunsCollection;
  tag: typeof tagsCollection;
};

const collections: CollectionsByEntity = {
  workflow_run: workflowRunsCollection,
  job_run: jobRunsCollection,
  tag: tagsCollection,
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
  deleteCollectionItem(collections[entity], id);
}

function applyUpsert(payload: SyncUpsert) {
  switch (payload.entity) {
    case "workflow_run":
      upsertCollectionItem(workflowRunsCollection, payload.data);
      break;
    case "job_run":
      upsertCollectionItem(jobRunsCollection, payload.data);
      break;
    case "tag":
      upsertCollectionItem(tagsCollection, payload.data);
      break;
    default:
      assertNever(payload);
  }
}

export async function startSync() {
  try {
    const result = await commands.sync(SyncChannel);
    if (result.status === "error") {
      console.error("sync command failed", result.error);
    }
  } catch (error) {
    console.error("sync connection failed", error);
  }
}
