import { useCallback } from "react";
import {
  Background,
  BackgroundVariant,
  Controls,
  ReactFlow,
  ReactFlowProvider,
  Node,
  Edge,
  type NodeMouseHandler,
  type OnInit,
} from "@xyflow/react";
import { useWorkflowXyFlowSchema } from "../hooks/useWorkflowXyFlowSchema";
import { useNodeXyTypes } from "../hooks/useNodeXyTypes";
import { useTheme } from "@/components/theme/theme-provider";
import { useCanvasStore } from "../hooks/useCanvasStore";

const DEFAULT_MAX_ZOOM = 1;

/**
 * Inner component that handles the ReactFlow canvas.
 * Must be inside ReactFlowProvider to access ReactFlow context.
 */
function WorkflowCanvasInner({
  initialNodes,
  initialEdges,
  runId,
  className,
}: {
  initialNodes: Node[];
  initialEdges: Edge[];
  runId?: string;
  className?: string;
}) {
  const nodeTypes = useNodeXyTypes();
  const { theme } = useTheme();

  // Store state and actions
  const nodes = useCanvasStore((state) => state.nodes);
  const edges = useCanvasStore((state) => state.edges);
  const onNodesChange = useCanvasStore((state) => state.onNodesChange);
  const onEdgesChange = useCanvasStore((state) => state.onEdgesChange);
  const init = useCanvasStore((state) => state.init);
  const selectNode = useCanvasStore((state) => state.selectNode);

  const handleInit: OnInit = useCallback(
    (reactFlowInstance) => {
      init({
        nodes: initialNodes,
        edges: initialEdges,
        runId,
        fitView: () => reactFlowInstance.fitView({ maxZoom: DEFAULT_MAX_ZOOM }),
      });
    },
    [initialNodes, initialEdges, runId, init]
  );

  const handleNodeClick: NodeMouseHandler = useCallback(
    (_event, node) => {
      const nodeType = node.type;
      if (!nodeType) return;

      const nodeData = node.data as {
        label?: string;
        jobId?: string;
        channelId?: string;
      };
      const id = nodeType === "job" ? nodeData.jobId : nodeData.channelId;

      if (id && (nodeType === "job" || nodeType === "channel")) {
        selectNode(id);
      }
    },
    [selectNode]
  );

  return (
    <ReactFlow
      nodes={nodes}
      edges={edges}
      onInit={handleInit}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onNodeClick={handleNodeClick}
      nodeTypes={nodeTypes}
      className={className}
      colorMode={theme}
      nodesDraggable={false}
      proOptions={{
        hideAttribution: true,
      }}
    >
      <Background variant={BackgroundVariant.Dots} gap={12} size={1} />
      <Controls
        position="bottom-right"
        style={{ marginRight: 16, marginBottom: 16 }}
      />
    </ReactFlow>
  );
}

type WorkflowCanvasProps = {
  workflowVersionId: string;
  runId?: string;
  className?: string;
};

export function WorkflowCanvas({
  workflowVersionId,
  runId,
  className,
}: WorkflowCanvasProps) {
  const schema = useWorkflowXyFlowSchema(workflowVersionId, runId);
  if (!schema) return null;

  return (
    <ReactFlowProvider>
      <WorkflowCanvasInner
        initialNodes={schema.nodes}
        initialEdges={schema.edges}
        runId={schema.runId}
        className={className}
      />
    </ReactFlowProvider>
  );
}
