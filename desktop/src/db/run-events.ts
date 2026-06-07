import {
  createCollection,
  localOnlyCollectionOptions,
} from "@tanstack/react-db";
import { Event } from "@/bindings";

export const runEventsCollection = createCollection(
  localOnlyCollectionOptions<Event>({
    id: "run-events",
    getKey: (item) => item.id,
  })
);
