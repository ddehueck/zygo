import {
  createCollection,
  localOnlyCollectionOptions,
} from "@tanstack/react-db";
import { Run } from "@/bindings";

type WithWorkflowId<T> = T & {
  workflow_id: string;
};

export const runsCollection = createCollection(
  localOnlyCollectionOptions<WithWorkflowId<Run>>({
    id: "runs",
    getKey: (item) => item.id,
  })
);
