import { eq, useLiveQuery } from "@tanstack/react-db";
import { useMemo, useState, type ReactNode } from "react";
import type { Selection } from "react-aria-components/Table";

import { Description, Heading, Text } from "@/components/Text";
import {
  dataReferencesCollection,
  jobRunsCollection,
  tagsCollection,
} from "@/db/collections";
import { formatByteSize } from "../lib/dummy-metadata";
import { buildDataTree, findDataTreeNode, flattenDataTree } from "../lib/build-data-tree";
import { presentDataRow } from "../lib/data-lookups";
import { DataTreeIndex } from "../lib/data-tree-index";
import { DataReferenceDetails } from "./DataReferenceDetails";
import { DataTreeTable } from "./DataTreeTable";

type DataViewerProps = {
  workflowRunId: number;
  workflowId: string;
};

export function DataViewer({ workflowRunId, workflowId }: DataViewerProps) {
  const referencesQuery = useLiveQuery({
    query: (q) =>
      q
        .from({ reference: dataReferencesCollection })
        .where(({ reference }) => eq(reference.workflow_run_id, workflowRunId)),
  });
  const jobsQuery = useLiveQuery({
    query: (q) =>
      q
        .from({ jobRun: jobRunsCollection })
        .where(({ jobRun }) => eq(jobRun.workflow_run_id, workflowRunId)),
  });
  const tagsQuery = useLiveQuery({
    query: (q) =>
      q
        .from({ tag: tagsCollection })
        .where(({ tag }) => eq(tag.workflow_run_id, workflowRunId)),
  });

  const index = useMemo(
    () => new DataTreeIndex(referencesQuery.data, jobsQuery.data, tagsQuery.data),
    [referencesQuery.data, jobsQuery.data, tagsQuery.data],
  );
  const roots = useMemo(() => buildDataTree(index), [index]);

  const [selectedKeys, setSelectedKeys] = useState<Selection>(new Set());

  const selectedId =
    selectedKeys === "all"
      ? null
      : [...selectedKeys].map((key) => Number(key)).find((id) => Number.isFinite(id)) ?? null;
  const selectedNode = selectedId === null ? null : findDataTreeNode(roots, selectedId);

  const isLoading =
    referencesQuery.isLoading || jobsQuery.isLoading || tagsQuery.isLoading;
  const isError = referencesQuery.isError || jobsQuery.isError || tagsQuery.isError;

  if (isLoading) {
    return (
      <ViewerShell workflowId={workflowId}>
        <Text variant="muted">Loading data references…</Text>
      </ViewerShell>
    );
  }

  if (isError) {
    return (
      <ViewerShell workflowId={workflowId}>
        <Text role="alert" size="small" variant="danger">
          Unable to load data for this workflow run.
        </Text>
      </ViewerShell>
    );
  }

  const allNodes = flattenDataTree(roots);
  const totalBytes = allNodes.reduce((sum, node) => {
    const reference = index.referencesById.get(node.id);
    return reference ? sum + presentDataRow(reference).sizeBytes : sum;
  }, 0);
  const inputCount = roots.length;

  return (
    <div className="flex min-h-0 w-full flex-1 flex-col overflow-hidden">
      <header className="shrink-0 border-b border-app-border px-6 py-5">
        <Heading size="medium">Data</Heading>
        <Description className="mt-1">
          Lineage for <span className="font-mono">{workflowId}</span>
          {" · "}
          {inputCount} input{inputCount === 1 ? "" : "s"}
          {" · "}
          {allNodes.length} file{allNodes.length === 1 ? "" : "s"}
          {" · "}
          {formatByteSize(totalBytes)}
        </Description>
      </header>

      <div className="flex min-h-0 flex-1 flex-col lg:flex-row">
        <div className="min-h-0 min-w-0 flex-1 overflow-auto">
          <DataTreeTable
            key={workflowRunId}
            roots={roots}
            index={index}
            selectedKeys={selectedKeys}
            onSelectionChange={setSelectedKeys}
          />
        </div>
        {selectedNode ? (
          <DataReferenceDetails
            node={selectedNode}
            index={index}
            workflowRunId={String(workflowRunId)}
          />
        ) : (
          <aside className="hidden border-l border-app-border bg-app-bg-surface px-5 py-8 text-center lg:flex lg:w-80 lg:shrink-0 lg:flex-col lg:items-center lg:justify-center">
            <Text size="small" variant="muted">
              Select a file to inspect its URI, producing job, and downstream outputs.
            </Text>
          </aside>
        )}
      </div>
    </div>
  );
}

function ViewerShell({
  workflowId,
  children,
}: {
  workflowId: string;
  children: ReactNode;
}) {
  return (
    <div className="mx-auto w-full max-w-5xl px-6 py-10">
      <Heading size="medium">Data</Heading>
      <Description className="mt-1">
        Lineage for <span className="font-mono">{workflowId}</span>
      </Description>
      <div className="mt-8">{children}</div>
    </div>
  );
}
