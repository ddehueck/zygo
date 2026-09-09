import type { Key } from "react-aria-components/Table";

import { DataTreeIndex } from "./data-tree-index";

export type DataTreeNode = {
  id: number;
  children: DataTreeNode[];
};

/**
 * Materializes a lineage tree from a {@link DataTreeIndex}.
 *
 * Nodes are graph-only (`id` + `children`). Look up references/jobs/tags to render rows.
 */
export function buildDataTree(index: DataTreeIndex): DataTreeNode[] {
  const visiting = new Set<number>();
  const roots: DataTreeNode[] = [];
  for (const id of sortedIds(index.rootIds, index)) {
    roots.push(materializeNode(id, index, visiting));
  }
  return roots;
}

function materializeNode(id: number, index: DataTreeIndex, visiting: Set<number>): DataTreeNode {
  const node: DataTreeNode = { id, children: [] };
  if (visiting.has(id)) return node;

  visiting.add(id);
  for (const childId of sortedIds(index.childIds(id), index)) {
    node.children.push(materializeNode(childId, index, visiting));
  }
  visiting.delete(id);

  return node;
}

function sortedIds(ids: readonly number[], index: DataTreeIndex): number[] {
  return [...ids].sort((a, b) => index.compareIds(a, b));
}

export function flattenDataTree(nodes: DataTreeNode[]): DataTreeNode[] {
  const result: DataTreeNode[] = [];

  function walk(node: DataTreeNode) {
    result.push(node);
    for (const child of node.children) {
      walk(child);
    }
  }

  for (const node of nodes) {
    walk(node);
  }

  return result;
}

export function findDataTreeNode(nodes: DataTreeNode[], id: number): DataTreeNode | null {
  for (const node of nodes) {
    if (node.id === id) return node;
    const nested = findDataTreeNode(node.children, id);
    if (nested) return nested;
  }
  return null;
}

// IDs of nodes that should start expanded so the tree is open through `maxDepth` levels.
export function collectExpandedKeysUpToDepth(nodes: DataTreeNode[], maxDepth: number): Set<Key> {
  const keys = new Set<Key>();

  function walk(node: DataTreeNode, depth: number) {
    if (depth >= maxDepth || node.children.length === 0) return;
    keys.add(node.id);
    for (const child of node.children) {
      walk(child, depth + 1);
    }
  }

  for (const node of nodes) {
    walk(node, 1);
  }

  return keys;
}
