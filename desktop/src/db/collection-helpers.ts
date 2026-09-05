import type { Collection, UtilsRecord } from "@tanstack/db";

type Entity = { id: string };

type MutableEntityCollection<T extends Entity> = Pick<
  Collection<T, string, UtilsRecord, never, T>,
  "has" | "insert" | "update"
>;

type DeletableCollection = {
  delete(id: string): unknown;
  has(id: string): boolean;
};

export function upsertCollectionItem<T extends Entity>(
  collection: MutableEntityCollection<T>,
  item: T,
) {
  if (collection.has(item.id)) {
    collection.update(item.id, (draft) => Object.assign(draft, item));
  } else {
    collection.insert(item);
  }
}

export function deleteCollectionItem(collection: DeletableCollection, id: string) {
  if (collection.has(id)) {
    collection.delete(id);
  }
}
