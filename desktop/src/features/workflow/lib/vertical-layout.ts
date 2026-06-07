import { Node, Edge } from "@xyflow/react";
import Dagre from "@dagrejs/dagre";

const NODE_WIDTH = 150;
const NODE_HEIGHT = 40;

type LayoutDirection = "TB" | "LR";

/**
 * Applies Dagre layout to nodes and edges.
 * @param nodes - The nodes to layout
 * @param edges - The edges connecting nodes
 * @param direction - "TB" for vertical (top-to-bottom), "LR" for horizontal (left-to-right)
 * @returns Nodes with updated positions and the original edges
 * @docs https://reactflow.dev/learn/layouting/layouting
 */
export function getLayoutedElements(
  nodes: Node[],
  edges: Edge[],
  direction: LayoutDirection = "TB"
): { nodes: Node[]; edges: Edge[] } {
  const g = new Dagre.graphlib.Graph().setDefaultEdgeLabel(() => ({}));
  g.setGraph({
    rankdir: direction,
    align: undefined, // Center nodes horizontally within their rank
    ranksep: 50, // Vertical spacing between ranks
    nodesep: 30, // Horizontal spacing between nodes in same rank
  });

  edges.forEach((edge) => g.setEdge(edge.source, edge.target));
  nodes.forEach((node) =>
    g.setNode(node.id, {
      ...node,
      width: node.measured?.width ?? NODE_WIDTH,
      height: node.measured?.height ?? NODE_HEIGHT,
    })
  );

  Dagre.layout(g);

  return {
    nodes: nodes.map((node) => {
      const position = g.node(node.id);
      // Shift dagre node position (anchor=center) to top-left to match React Flow anchor
      const x = position.x - (node.measured?.width ?? NODE_WIDTH) / 2;
      const y = position.y - (node.measured?.height ?? NODE_HEIGHT) / 2;

      return { ...node, position: { x, y } };
    }),
    edges,
  };
}
