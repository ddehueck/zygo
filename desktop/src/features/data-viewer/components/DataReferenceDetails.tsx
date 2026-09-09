import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";

import { Icon, iconDefinitions } from "@/components/icons";
import { Description, Heading, Text } from "@/components/Text";
import { formatDate } from "@/lib/dates";
import { TagBadge } from "@/features/workflow-runs/components/TagBadge";
import { RunStatus } from "@/features/workflow-runs/components/statuses";

import type { DataTreeNode } from "../lib/build-data-tree";
import { getProducedByJob, presentDataRow } from "../lib/data-lookups";
import type { DataTreeIndex } from "../lib/data-tree-index";

type DataReferenceDetailsProps = {
  node: DataTreeNode;
  index: DataTreeIndex;
  workflowRunId: string;
};

export function DataReferenceDetails({
  node,
  index,
  workflowRunId,
}: DataReferenceDetailsProps) {
  const reference = index.referencesById.get(node.id);
  if (!reference) return null;

  const { fileName, kind, sizeLabel } = presentDataRow(reference);
  const producedByJob = getProducedByJob(reference, index.jobsById);
  const tags = index.tagsByRefId.get(node.id) ?? [];

  return (
    <aside
      aria-label={`Details for ${fileName}`}
      className="flex h-full min-h-0 w-full flex-col overflow-y-auto border-l border-app-border bg-app-bg-surface px-5 py-5 lg:max-w-sm lg:shrink-0"
    >
      <div className="flex min-w-0 items-start gap-3">
        <span className="mt-0.5 flex size-9 shrink-0 items-center justify-center rounded-lg border border-app-border bg-app-bg-elevated">
          <Icon
            aria-hidden
            className="size-4 text-app-foreground-secondary"
            definition={iconDefinitions.file}
          />
        </span>
        <div className="min-w-0 flex-1">
          <Heading size="medium" className="text-lg">
            {fileName}
          </Heading>
          <Description className="mt-1">
            {kind} · {sizeLabel}
          </Description>
        </div>
      </div>

      <dl className="mt-6 space-y-4">
        <DetailField label="URI">
          <Text size="small" className="block break-all font-mono">
            {reference.uri}
          </Text>
        </DetailField>

        <DetailField label="Created">{formatDate(reference.created_at)}</DetailField>

        <DetailField label="Role">
          {reference.source_job_run_id === null ? "Workflow input" : "Job output"}
          {reference.is_replay ? " · Replay" : ""}
        </DetailField>

        <DetailField label="Produced by">
          {producedByJob ? (
            <span className="flex flex-col items-start gap-2">
              <Link
                to="/runs/$workflowRunId/jobs/$jobId"
                params={{ workflowRunId, jobId: String(producedByJob.id) }}
                className="font-mono text-sm font-medium text-app-accent hover:underline"
              >
                {producedByJob.job_id}
              </Link>
              <RunStatus status={producedByJob.status} />
            </span>
          ) : (
            <Text size="small" variant="muted">
              External input
            </Text>
          )}
        </DetailField>

        <DetailField label="Storage">
          <Text size="small" variant="muted">
            Object store metadata coming soon
          </Text>
          <Text size="small" className="mt-1 block">
            Region us-east-1 · Standard
          </Text>
        </DetailField>

        {tags.length > 0 && (
          <DetailField label="Tags">
            <div className="flex flex-wrap gap-1.5">
              {tags.map((tag) => (
                <TagBadge key={tag.id} value={tag.value} includeIcon />
              ))}
            </div>
          </DetailField>
        )}
      </dl>

      {node.children.length > 0 && (
        <section className="mt-8 border-t border-app-border pt-5">
          <Heading size="medium" className="text-base">
            Downstream outputs
          </Heading>
          <Description className="mt-1">
            {node.children.length} file{node.children.length === 1 ? "" : "s"} produced from jobs that
            consumed this input
          </Description>
          <ul className="mt-3 space-y-2">
            {node.children.map((child) => {
              const childReference = index.referencesById.get(child.id);
              const childName = childReference
                ? presentDataRow(childReference).fileName
                : String(child.id);

              return (
                <li key={child.id} className="flex min-w-0 items-center gap-2 text-sm">
                  <Icon
                    aria-hidden
                    className="size-3.5 shrink-0 text-app-foreground-muted"
                    definition={iconDefinitions.file}
                  />
                  <span className="min-w-0 truncate font-medium text-app-foreground">
                    {childName}
                  </span>
                </li>
              );
            })}
          </ul>
        </section>
      )}
    </aside>
  );
}

function DetailField({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div>
      <dt>
        <Text size="small" variant="muted" className="block">
          {label}
        </Text>
      </dt>
      <dd className="mt-1 text-sm text-app-foreground">{children}</dd>
    </div>
  );
}
