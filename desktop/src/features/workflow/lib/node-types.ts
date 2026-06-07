/**
 * Node type definitions for the workflow canvas.
 * This module defines the types and metadata for different node kinds
 * that can appear in a workflow graph.
 */

// ================================
// NODE KIND TYPE
// ================================

/**
 * The kind of node in the workflow graph.
 * This union type ensures type safety when working with different node types.
 */
export type NodeKind = "job" | "channel";

// ================================
// NODE METADATA
// ================================

export type NodeMetadata = {
  /** Display name for the node type */
  displayName: string;
  /** Description of what this node type represents */
  description: string;
};

/**
 * Registry of metadata for each node kind.
 * Used for consistent display across the UI.
 */
export const NODE_METADATA: Record<NodeKind, NodeMetadata> = {
  job: {
    displayName: "Job",
    description: "A computational task in the workflow",
  },
  channel: {
    displayName: "Channel",
    description: "A data channel receiving and broadcasting data",
  },
};

// ================================
// TYPE GUARDS
// ================================

/**
 * Type guard to check if a string is a valid NodeKind
 */
export function isNodeKind(kind: string): kind is NodeKind {
  return kind === "job" || kind === "channel";
}

// ================================
// UTILITY FUNCTIONS
// ================================

/**
 * Get the display name for a node kind
 */
export function getNodeDisplayName(kind: NodeKind): string {
  return NODE_METADATA[kind].displayName;
}

/**
 * Get the description for a node kind
 */
export function getNodeDescription(kind: NodeKind): string {
  return NODE_METADATA[kind].description;
}
