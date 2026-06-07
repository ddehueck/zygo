import type { NodeTypes } from "@xyflow/react";
import { JobNode } from "../components/nodes/JobNode";
import { ChannelNode } from "../components/nodes/ChannelNode";
import type { NodeKind } from "../lib/node-types";

/**
 * Registry mapping node kinds to their ReactFlow components.
 * This is defined outside of components to prevent re-renders.
 */
const nodeTypes: Record<NodeKind, NodeTypes[string]> = {
  job: JobNode,
  channel: ChannelNode,
};

/**
 * Hook that returns the ReactFlow node types registry.
 * Uses the NodeKind type from lib/node-types.ts for type safety.
 */
export function useNodeXyTypes(): NodeTypes {
  return nodeTypes;
}
