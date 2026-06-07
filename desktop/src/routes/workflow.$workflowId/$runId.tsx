import { createFileRoute, Link } from "@tanstack/react-router";
import { WorkflowCanvas } from "@/features/workflow/components/WorkflowCanvas";
import { eq, useLiveQuery } from "@tanstack/react-db";
import { runsCollection } from "@/db/runs";
import { useSelectedNode } from "@/features/workflow/hooks/useSelectedNode";
import { JobEventsSidebar } from "@/features/run/components/JobEventsSidebar";
import { ChannelEventsSidebar } from "@/features/run/components/ChannelEventsSidebar";

export const Route = createFileRoute("/workflow/$workflowId/$runId")({
  component: RunDetailPage,
});

function RunDetailPage() {
  const { workflowId, runId } = Route.useParams();

  const { data: run, isLoading } = useLiveQuery(
    (q) =>
      q
        .from({ runs: runsCollection })
        .where(({ runs }) => eq(runs.workflow_id, workflowId))
        .where(({ runs }) => eq(runs.id, runId))
        .findOne(),
    [runId, workflowId]
  );

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        Loading...
      </div>
    );
  }

  if (!run) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <p className="text-muted-foreground">Run not found</p>
          <Link
            to="/workflow/$workflowId"
            params={{ workflowId }}
            className="mt-2 text-sm text-primary hover:underline"
          >
            Return to workflow
          </Link>
        </div>
      </div>
    );
  }

  return (
    <RunDetailContent
      workflowVersionId={run.workflow_version_id}
      runId={runId}
    />
  );
}

function RunDetailContent({
  workflowVersionId,
  runId,
}: {
  workflowVersionId: string;
  runId: string;
}) {
  const selectedNode = useSelectedNode(workflowVersionId);

  return (
    <div className="flex h-full w-full">
      {/* Canvas Area */}
      <div className="flex-1 overflow-hidden">
        <WorkflowCanvas
          workflowVersionId={workflowVersionId}
          runId={runId}
          className="h-full w-full"
        />
      </div>

      {/* Right Sidebar - Job Events */}
      {selectedNode?.kind === "job" && (
        <JobEventsSidebar
          runId={runId}
          jobId={selectedNode.id}
          jobName={selectedNode.name}
        />
      )}

      {/* Right Sidebar - Channel Events */}
      {selectedNode?.kind === "channel" && (
        <ChannelEventsSidebar
          runId={runId}
          channelId={selectedNode.id}
          channelName={selectedNode.name}
        />
      )}
    </div>
  );
}
