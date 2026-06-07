import { useEffect, useRef, useState } from "react";
import {
  Node,
  Edge,
  useNodesState,
  useEdgesState,
  useNodesInitialized,
  useReactFlow,
  OnNodesChange,
  OnEdgesChange,
} from "@xyflow/react";
import { getLayoutedElements } from "../lib/vertical-layout";

const DEFAULT_MAX_ZOOM = 1;

type UseLayoutedNodesReturn = {
  nodes: Node[];
  edges: Edge[];
  onNodesChange: OnNodesChange;
  onEdgesChange: OnEdgesChange;
};

/**
 * Manages nodes and edges state with automatic Dagre layout after measurement.
 * Must be used inside a ReactFlowProvider (needs ReactFlow context).
 *
 * This hook:
 * 1. Initializes nodes/edges state
 * 2. Waits for React Flow to measure the DOM nodes
 * 3. Applies Dagre layout using actual measured dimensions
 * 4. Fits the view after layout is applied
 * 5. Resets state when initialNodes/initialEdges change (e.g., different workflow version)
 * 6. Updates node data (e.g., runId) without re-layout when only runId changes
 */
export function useLayoutedNodes(
  initialNodes: Node[],
  initialEdges: Edge[],
  runId?: string
): UseLayoutedNodesReturn {
  const { fitView } = useReactFlow();

  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
  const [layoutApplied, setLayoutApplied] = useState(false);

  // Track previous initial values to detect changes
  const prevInitialNodesRef = useRef(initialNodes);
  const prevInitialEdgesRef = useRef(initialEdges);
  const prevRunIdRef = useRef(runId);

  const nodesInitialized = useNodesInitialized();

  // Reset state when initialNodes/initialEdges change (structural change)
  useEffect(() => {
    if (
      prevInitialNodesRef.current !== initialNodes ||
      prevInitialEdgesRef.current !== initialEdges
    ) {
      setNodes(initialNodes);
      setEdges(initialEdges);
      setLayoutApplied(false);
      prevInitialNodesRef.current = initialNodes;
      prevInitialEdgesRef.current = initialEdges;
    }
  }, [initialNodes, initialEdges, setNodes, setEdges]);

  // Update node data when runId changes (without re-layout)
  useEffect(() => {
    if (prevRunIdRef.current !== runId && layoutApplied) {
      setNodes((currentNodes) =>
        currentNodes.map((node) => ({
          ...node,
          data: { ...node.data, runId },
        }))
      );
      prevRunIdRef.current = runId;
    }
  }, [runId, layoutApplied, setNodes]);

  // Apply layout once nodes are measured
  useEffect(() => {
    if (nodesInitialized && !layoutApplied && nodes.length > 0) {
      // Check that all nodes have been measured
      const allMeasured = nodes.every(
        (node) => node.measured?.width && node.measured?.height
      );

      if (allMeasured) {
        const { nodes: layoutedNodes } = getLayoutedElements(
          nodes,
          edges,
          "TB"
        );
        // Apply layout and add runId to node data
        setNodes(
          layoutedNodes.map((node) => ({
            ...node,
            data: { ...node.data, runId },
          }))
        );
        setLayoutApplied(true);
        prevRunIdRef.current = runId;

        // Fit view after layout is applied
        window.requestAnimationFrame(() => {
          fitView({ maxZoom: DEFAULT_MAX_ZOOM });
        });
      }
    }
  }, [nodesInitialized, layoutApplied, nodes, edges, runId, setNodes, fitView]);

  return { nodes, edges, onNodesChange, onEdgesChange };
}
