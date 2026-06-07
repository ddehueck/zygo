import { useGetWorkflowVersionSchema } from "@/api";
import { Node, Edge } from "@xyflow/react";
import { useMemo } from "react";
import { createOrderMap } from "../lib/order";
import "@xyflow/react/dist/style.css";

type WorkflowXyFlowSchema = {
  nodes: Node[];
  edges: Edge[];
  runId?: string;
};

/**
 * Creates the initial nodes and edges from the workflow schema.
 * Note: Nodes are created with position {x: 0, y: 0} - layout should be applied
 * after React Flow has measured the nodes using useLayoutedNodes hook.
 *
 * The schema (nodes/edges) is memoized separately from runId to prevent
 * re-layout when only runId changes for status updates.
 *
 * @param workflowVersionId - The workflow version to fetch schema for
 * @param runId - Optional run ID to enable status indicators on nodes
 */
export function useWorkflowXyFlowSchema(
  workflowVersionId: string,
  runId?: string
): WorkflowXyFlowSchema | null {
  const { data: workflowVersionSchema } = useGetWorkflowVersionSchema({
    workflow_version_id: workflowVersionId,
  });

  // Memoize the structural schema separately from runId
  // This prevents re-layout when only runId changes
  const schema = useMemo(() => {
    if (!workflowVersionSchema) return null;

    // Compute topological order for nodes
    const orderMap = createOrderMap(
      workflowVersionSchema.jobs,
      workflowVersionSchema.channels,
      workflowVersionSchema.edges
    );

    // Track which nodes have incoming/outgoing edges
    const hasIncoming = new Set<string>();
    const hasOutgoing = new Set<string>();

    workflowVersionSchema.edges.forEach((edge) => {
      if (edge.kind === "input") {
        // channel -> job: channel has outgoing, job has incoming
        hasOutgoing.add(edge.channel_id);
        hasIncoming.add(edge.job_id);
      } else {
        // job -> channel: job has outgoing, channel has incoming
        hasOutgoing.add(edge.job_id);
        hasIncoming.add(edge.channel_id);
      }
    });

    const jobNodes: Node[] = workflowVersionSchema.jobs.map((job) => ({
      id: job.id,
      type: "job",
      position: { x: 0, y: 0 }, // Will be set by layout after measurement
      data: {
        label: job.name,
        jobId: job.id,
        order: orderMap.get(job.id) ?? 0,
        hasIncoming: hasIncoming.has(job.id),
        hasOutgoing: hasOutgoing.has(job.id),
      },
    }));

    const channelNodes: Node[] = workflowVersionSchema.channels.map(
      (channel) => ({
        id: channel.id,
        type: "channel",
        position: { x: 0, y: 0 }, // Will be set by layout after measurement
        data: {
          label: channel.name,
          channelId: channel.id,
          order: orderMap.get(channel.id) ?? 0,
          hasIncoming: hasIncoming.has(channel.id),
          hasOutgoing: hasOutgoing.has(channel.id),
        },
      })
    );

    const nodes = jobNodes.concat(channelNodes);

    // Create edges with correct direction based on edge kind:
    // - "input" edge: channel -> job (data flows from channel to job)
    // - "output" edge: job -> channel (data flows from job to channel)
    // Specify sourceHandle and targetHandle to force top/bottom connections
    const edges: Edge[] = workflowVersionSchema.edges.map((edge) => ({
      id: `${edge.job_id}::${edge.channel_id}::${edge.kind}`,
      source: edge.kind === "input" ? edge.channel_id : edge.job_id,
      target: edge.kind === "input" ? edge.job_id : edge.channel_id,
      sourceHandle: "bottom",
      targetHandle: "top",
    }));

    return { nodes, edges };
  }, [workflowVersionSchema]);

  if (!schema) return null;

  return { ...schema, runId };
}
