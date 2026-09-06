import type { Collection, UtilsRecord } from "@tanstack/db";

type Entity = { public_id: string } | { id: number };

type MutableEntityCollection<T extends Entity, K extends string | number> = Pick<
  Collection<T, K, UtilsRecord, never, T>,
  "has" | "insert" | "update"
>;

type DeletableCollection<K extends string | number> = {
  delete(id: K): unknown;
  has(id: K): boolean;
};

export function upsertCollectionItem<T extends Entity, K extends string | number>(
  collection: MutableEntityCollection<T, K>,
  item: T & ({ public_id: K } | { id: K }),
) {
  const id = ("public_id" in item ? item.public_id : item.id) as K;
  if (collection.has(id)) {
    collection.update(id, (draft) => Object.assign(draft, item));
  } else {
    collection.insert(item);
  }
}

export function deleteCollectionItem<K extends string | number>(
  collection: DeletableCollection<K>,
  id: K,
) {
  if (collection.has(id)) {
    collection.delete(id);
  }
}
