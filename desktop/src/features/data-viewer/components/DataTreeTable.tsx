import { Collection, type Selection } from "react-aria-components/Table";
import { useState } from "react";

import { Icon, iconDefinitions } from "@/components/icons";
import { IconButton } from "@/components/IconButton";
import { Cell, Column, Row, Table, TableBody, TableHeader } from "@/components/Table";
import { Text } from "@/components/Text";
import { formatDate } from "@/lib/dates";

import { collectExpandedKeysUpToDepth, type DataTreeNode } from "../lib/build-data-tree";
import { getProducedByJob, presentDataRow } from "../lib/data-lookups";
import type { DataTreeIndex } from "../lib/data-tree-index";
import { localPathFromUri, revealLocalUri } from "../lib/reveal-local-path";

const INITIAL_EXPAND_DEPTH = 10;

const columnClassName =
  "h-auto border-b border-app-border py-0 text-xs font-normal tracking-normal text-app-foreground-muted [&_[role=presentation]]:h-auto [&_[role=presentation]]:px-3 [&_[role=presentation]]:py-1.5";

type DataTreeTableProps = {
  roots: DataTreeNode[];
  index: DataTreeIndex;
  selectedKeys: Selection;
  onSelectionChange: (keys: Selection) => void;
};

export function DataTreeTable({
  roots,
  index,
  selectedKeys,
  onSelectionChange,
}: DataTreeTableProps) {
  const [expandedKeys, setExpandedKeys] = useState<Selection>(() =>
    collectExpandedKeysUpToDepth(roots, INITIAL_EXPAND_DEPTH),
  );

  return (
    <Table
      aria-label="Data lineage"
      treeColumn="name"
      selectionMode="single"
      selectionBehavior="replace"
      disallowEmptySelection
      selectedKeys={selectedKeys}
      onSelectionChange={onSelectionChange}
      expandedKeys={expandedKeys}
      onExpandedChange={setExpandedKeys}
      className="min-w-180"
    >
      <TableHeader className="bg-app-bg-surface backdrop-blur-none">
        <Column id="name" isRowHeader defaultWidth="2fr" minWidth={220} className={columnClassName}>
          Name
        </Column>
        <Column id="kind" defaultWidth={100} minWidth={88} className={columnClassName}>
          Type
        </Column>
        <Column id="producedBy" defaultWidth="1fr" minWidth={140} className={columnClassName}>
          Produced by
        </Column>
        <Column id="size" defaultWidth={90} minWidth={80} className={columnClassName}>
          Size
        </Column>
        <Column id="created" defaultWidth={160} minWidth={120} className={columnClassName}>
          Created
        </Column>
        <Column id="open" defaultWidth={72} minWidth={72} className={columnClassName}>
          Open
        </Column>
      </TableHeader>
      <TableBody items={roots} renderEmptyState={() => <EmptyState />}>
        {function renderNode(node: DataTreeNode) {
          const reference = index.referencesById.get(node.id);
          if (!reference) return null;

          const { fileName, kind, sizeLabel } = presentDataRow(reference);
          const producedByJob = getProducedByJob(reference, index.jobsById);

          return (
            <Row id={node.id} textValue={fileName}>
              <Cell>
                <span className="flex min-w-0 items-center gap-2">
                  <Icon
                    aria-hidden
                    className="size-3.5 shrink-0 text-app-foreground-muted"
                    definition={iconDefinitions.file}
                  />
                  <span className="min-w-0 truncate font-medium" title={fileName}>
                    {fileName}
                  </span>
                </span>
              </Cell>
              <Cell>
                <Text size="small" variant="muted">
                  {kind}
                </Text>
              </Cell>
              <Cell>
                {producedByJob ? (
                  <span className="truncate font-mono text-xs" title={producedByJob.job_id}>
                    {producedByJob.job_id}
                  </span>
                ) : (
                  <Text size="small" variant="muted">
                    Workflow input
                  </Text>
                )}
              </Cell>
              <Cell>
                <Text size="small" variant="muted" className="font-mono tabular-nums">
                  {sizeLabel}
                </Text>
              </Cell>
              <Cell>
                <Text size="small" variant="muted">
                  {formatDate(reference.created_at)}
                </Text>
              </Cell>
              <Cell>
                <IconButton
                  aria-label={`Open ${fileName} in directory`}
                  isDisabled={localPathFromUri(reference.uri) === null}
                  onPress={() => {
                    void revealLocalUri(reference.uri);
                  }}
                >
                  <Icon aria-hidden className="size-4" definition={iconDefinitions.open} />
                </IconButton>
              </Cell>
              <Collection items={node.children}>{renderNode}</Collection>
            </Row>
          );
        }}
      </TableBody>
    </Table>
  );
}

function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center gap-2 py-16 text-center">
      <Icon
        aria-hidden
        className="size-8 text-app-foreground-muted"
        definition={iconDefinitions.file}
      />
      <Text variant="muted" size="small">
        No data references for this run.
      </Text>
    </div>
  );
}
