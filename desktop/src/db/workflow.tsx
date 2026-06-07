import {
  createCollection,
  localOnlyCollectionOptions,
} from "@tanstack/react-db";
import { Workflow } from "@/bindings";

export const workflowCollection = createCollection(
  localOnlyCollectionOptions<Workflow>({
    id: "workflows",
    getKey: (item) => item.id,
  })
);
