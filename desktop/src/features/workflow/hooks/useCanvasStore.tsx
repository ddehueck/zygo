import { create } from "zustand";
import type { Node, Edge, OnNodesChange, OnEdgesChange } from "@xyflow/react";
import { applyNodeChanges, applyEdgeChanges } from "@xyflow/react";
import { getLayoutedElements } from "../lib/vertical-layout";

// ================================
// TYPES
// ================================

type InitParams = {
  nodes: Node[];
  edges: Edge[];
  runId?: string;
  fitView: () => void;
};

type CanvasState = {
  nodes: Node[];
  edges: Edge[];
  layoutApplied: boolean;
  selectedNodeId: string | null;
  runId: string | null;
  fitView: (() => void) | null;
};

type CanvasActions = {
  init: (params: InitParams) => void;
  onNodesChange: OnNodesChange;
  onEdgesChange: OnEdgesChange;
  selectNode: (nodeId: string) => void;
  getSelectedNode: () => Node | undefined;
  updateRunId: (runId: string) => void;
  reset: () => void;
};

type CanvasStore = CanvasState & CanvasActions;

// ================================
// INITIAL STATE
// ================================

const initialState: CanvasState = {
  nodes: [],
  edges: [],
  layoutApplied: false,
  selectedNodeId: null,
  runId: null,
  fitView: null,
};

// ================================
// STORE
// ================================

export const useCanvasStore = create<CanvasStore>((set, get) => ({
  ...initialState,

  init: ({ nodes, edges, runId, fitView }) => {
    // Inject runId into each node's data so status indicators render
    const nodesWithRunId = runId
      ? nodes.map((node) => ({
          ...node,
          data: { ...node.data, runId },
        }))
      : nodes;

    set({
      nodes: nodesWithRunId,
      edges,
      runId: runId ?? null,
      fitView,
      layoutApplied: false,
      selectedNodeId: nodes[0]?.id ?? null,
    });
  },

  onNodesChange: (changes) => {
    let newNodes = applyNodeChanges(changes, get().nodes);
    // The first change is the layout change so once the nodes are measured, we can apply the layout
    if (changes.some((c) => c.type === "dimensions") && !get().layoutApplied) {
      const layout = getLayoutedElements(newNodes, get().edges, "TB");
      set({ nodes: layout.nodes, edges: layout.edges, layoutApplied: true });

      // fitView after layout so the graph is centered in the viewport
      requestAnimationFrame(() => {
        get().fitView?.();
      });
    } else {
      set({ nodes: newNodes });
    }
  },

  onEdgesChange: (changes) => {
    set((state) => ({
      edges: applyEdgeChanges(changes, state.edges),
    }));
  },

  getSelectedNode: () =>
    get().nodes.find((node) => node.id === get().selectedNodeId),

  selectNode: (nodeId: string) => {
    set({ selectedNodeId: nodeId });
  },

  updateRunId: (runId: string) => {
    const { nodes, layoutApplied } = get();
    if (!layoutApplied) return;

    set({
      runId,
      nodes: nodes.map((node) => ({
        ...node,
        data: { ...node.data, runId },
      })),
    });
  },

  reset: () => {
    set(initialState);
  },
}));

// ================================
// SELECTORS
// ================================

export const selectNodes = (state: CanvasStore) => state.nodes;
export const selectEdges = (state: CanvasStore) => state.edges;
export const selectLayoutApplied = (state: CanvasStore) => state.layoutApplied;
export const selectSelectedNodeId = (state: CanvasStore) =>
  state.selectedNodeId;
