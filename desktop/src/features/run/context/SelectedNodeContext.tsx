import {
  createContext,
  useContext,
  useState,
  useCallback,
  type ReactNode,
} from "react";
import type { NodeKind } from "@/features/workflow/lib/node-types";

// ================================
// TYPES
// ================================

export type SelectedNode = {
  kind: NodeKind;
  id: string;
  label: string;
};

type SelectedNodeContextValue = {
  selectedNode: SelectedNode | null;
  selectNode: (node: SelectedNode) => void;
  /** Initialize selection with the first node - should be called once on mount */
  initializeSelection: (node: SelectedNode) => void;
};

// ================================
// CONTEXT
// ================================

const SelectedNodeContext = createContext<SelectedNodeContextValue | null>(
  null
);

// ================================
// PROVIDER
// ================================

export function SelectedNodeProvider({ children }: { children: ReactNode }) {
  const [selectedNode, setSelectedNode] = useState<SelectedNode | null>(null);
  const [initialized, setInitialized] = useState(false);

  const selectNode = useCallback((node: SelectedNode) => {
    setSelectedNode(node);
  }, []);

  const initializeSelection = useCallback(
    (node: SelectedNode) => {
      if (!initialized) {
        setSelectedNode(node);
        setInitialized(true);
      }
    },
    [initialized]
  );

  return (
    <SelectedNodeContext.Provider
      value={{
        selectedNode,
        selectNode,
        initializeSelection,
      }}
    >
      {children}
    </SelectedNodeContext.Provider>
  );
}

// ================================
// HOOK
// ================================

export function useSelectedNode(): SelectedNodeContextValue {
  const context = useContext(SelectedNodeContext);
  if (!context) {
    throw new Error(
      "useSelectedNode must be used within a SelectedNodeProvider"
    );
  }
  return context;
}

