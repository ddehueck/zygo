import { useMemo } from "react";
import { useGetWorkflowVersionSchema } from "@/api";
import { useCanvasStore, selectSelectedNodeId } from "./useCanvasStore";
import type { Job, Channel } from "@/bindings";

// ================================
// TYPES
// ================================

export type SelectedJob = { kind: "job" } & Job;
export type SelectedChannel = { kind: "channel" } & Channel;
export type SelectedNode = SelectedJob | SelectedChannel | null;

// ================================
// HOOK
// ================================

/**
 * Hook that resolves the currently selected node ID to its job or channel data.
 * Uses the workflow schema to look up the full object by ID.
 *
 * @param workflowVersionId - The workflow version to fetch schema for
 * @returns The selected job or channel with a discriminated `kind` field, or null
 */
export function useSelectedNode(workflowVersionId: string): SelectedNode {
  const selectedNodeId = useCanvasStore(selectSelectedNodeId);
  const { data: schema } = useGetWorkflowVersionSchema({
    workflow_version_id: workflowVersionId,
  });

  return useMemo(() => {
    if (!selectedNodeId || !schema) return null;

    const job = schema.jobs.find((j) => j.id === selectedNodeId);
    if (job) return { kind: "job", ...job };

    const channel = schema.channels.find((c) => c.id === selectedNodeId);
    if (channel) return { kind: "channel", ...channel };

    return null;
  }, [selectedNodeId, schema]);
}
