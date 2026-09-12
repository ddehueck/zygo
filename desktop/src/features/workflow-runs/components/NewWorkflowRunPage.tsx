import { eq } from "@tanstack/db";
import { useLiveQuery } from "@tanstack/react-db";
import { useNavigate } from "@tanstack/react-router";
import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import type { Selection } from "react-aria-components/GridList";

import { commands, type CommandError } from "@/bindings";
import { Button } from "@/components/Button";
import { GridList, GridListItem } from "@/components/GridList";
import { Icon, iconDefinitions } from "@/components/icons";
import { Description, Heading, Text } from "@/components/Text";
import { workflowsCollection } from "@/db/collections";
import {
  acceptedInputFileExtensions,
  workflowJobOptions,
  type WorkflowJobOption,
} from "@/features/workflow-runs/lib/workflow-schema";
import { useFileDrop } from "@/hooks/use-file-drop";

type NewWorkflowRunPageProps = {
  workflowId: number;
};

type FileDropTargetProps = ReturnType<typeof useFileDrop>["dropTargetProps"];

const ENTIRE_WORKFLOW_KEY = "__entire_workflow__";

export function NewWorkflowRunPage({ workflowId }: NewWorkflowRunPageProps) {
  const navigate = useNavigate();
  const [inputPath, setInputPath] = useState<string | null>(null);
  const [isStarting, setIsStarting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [selectedScopeKey, setSelectedScopeKey] = useState<string>(ENTIRE_WORKFLOW_KEY);

  const { isDragging, dropTargetProps } = useFileDrop({
    onDrop: (paths) => {
      const [path] = paths;
      if (!path) return;
      setInputPath(path);
      setSubmitError(null);
    },
  });

  const workflowQuery = useLiveQuery({
    query: (q) =>
      q
        .from({ workflow: workflowsCollection })
        .where(({ workflow }) => eq(workflow.id, workflowId))
        .findOne(),
  });
  const workflow = workflowQuery.data;

  const selectedJobId =
    selectedScopeKey === ENTIRE_WORKFLOW_KEY ? null : selectedScopeKey;

  let jobs: WorkflowJobOption[] = [];
  let acceptedExtensions: string[] = [];
  let schemaError: string | null = null;
  if (workflow) {
    try {
      jobs = workflowJobOptions(workflow.schema);
      acceptedExtensions = acceptedInputFileExtensions(workflow.schema, selectedJobId);
    } catch {
      schemaError = "Unable to read this workflow's input schema.";
    }
  }

  const acceptLabel =
    acceptedExtensions.length > 0
      ? acceptedExtensions.map((extension) => `.${extension}`).join(", ")
      : "any file";

  async function chooseFile() {
    const filters =
      acceptedExtensions.length > 0
        ? [
            {
              name: "Workflow input",
              extensions: acceptedExtensions,
            },
          ]
        : undefined;

    const selected = await open({
      multiple: false,
      directory: false,
      filters,
    });

    if (typeof selected === "string") {
      setInputPath(selected);
      setSubmitError(null);
    }
  }

  async function startRun() {
    if (!inputPath) {
      setSubmitError(
        selectedJobId
          ? "Choose an input file to run this job."
          : "Choose an input file to run this workflow.",
      );
      return;
    }

    setIsStarting(true);
    setSubmitError(null);

    const result = await commands.startWorkflowRun({
      workflow_id: workflowId,
      input_paths: [inputPath],
      job_id: selectedJobId,
    });

    setIsStarting(false);

    if (result.status === "error") {
      setSubmitError(commandErrorMessage(result.error));
      return;
    }

    void navigate({
      to: "/runs/$workflowRunId",
      params: { workflowRunId: String(result.data.workflow_run_id) },
    });
  }

  function handleScopeChange(keys: Selection) {
    if (keys === "all") return;
    const [nextKey] = keys;
    if (typeof nextKey !== "string") return;
    setSelectedScopeKey(nextKey);
    setSubmitError(null);
  }

  if (workflowQuery.isLoading) {
    return (
      <main className="mx-auto w-full max-w-3xl px-6 py-10">
        <Text variant="muted">Loading workflow…</Text>
      </main>
    );
  }

  if (workflowQuery.isError) {
    return (
      <main className="mx-auto w-full max-w-3xl px-6 py-10">
        <Heading size="medium">Unable to load workflow</Heading>
        <Text className="mt-2 block" role="alert" variant="danger">
          Unable to load this workflow from the local sync store.
        </Text>
      </main>
    );
  }

  if (!workflow) {
    return (
      <main className="mx-auto w-full max-w-3xl px-6 py-10">
        <Heading size="medium">Workflow not found</Heading>
        <Description className="mt-2">
          This workflow may not be registered yet, or it was removed.
        </Description>
      </main>
    );
  }

  if (schemaError) {
    return (
      <main className="mx-auto w-full max-w-3xl px-6 py-10">
        <Heading size="medium">Unable to load workflow</Heading>
        <Text className="mt-2 block" role="alert" variant="danger">
          {schemaError}
        </Text>
      </main>
    );
  }

  const runLabel = selectedJobId ? "Run job" : "Run workflow";
  const inputDescription = selectedJobId
    ? `Provide an input file (${acceptLabel}) for job ${selectedJobId}.`
    : `Provide an input file (${acceptLabel}) to start a new run.`;

  return (
    <main className="mx-auto w-full max-w-3xl px-6 py-10">
      <header className="flex flex-wrap items-start justify-between gap-4">
        <div className="min-w-0">
          <Heading text={workflow.name} />
          <Description className="mt-2">{inputDescription}</Description>
        </div>
        <Button isPending={isStarting} onPress={() => void startRun()} isDisabled={!inputPath}>
          {runLabel}
        </Button>
      </header>

      <section className="mt-8">
        <div className="mb-3">
          <Text className="font-medium">What to run</Text>
          <Description className="mt-1">
            Start the full workflow, or isolate one job.
          </Description>
        </div>
        <RunScopePicker
          jobs={jobs}
          selectedKey={selectedScopeKey}
          onSelectionChange={handleScopeChange}
        />
      </section>

      <section className="mt-8">
        <div className="mb-3">
          <Text className="font-medium">Input file</Text>
        </div>
        <FileDropZone
          acceptLabel={acceptLabel}
          inputPath={inputPath}
          isDragging={isDragging}
          dropTargetProps={dropTargetProps}
          onBrowse={() => void chooseFile()}
          onClear={() => setInputPath(null)}
        />
        {submitError ? (
          <Text className="mt-3 block" role="alert" size="small" variant="danger">
            {submitError}
          </Text>
        ) : null}
      </section>
    </main>
  );
}

function RunScopePicker({
  jobs,
  selectedKey,
  onSelectionChange,
}: {
  jobs: WorkflowJobOption[];
  selectedKey: string;
  onSelectionChange: (keys: Selection) => void;
}) {
  return (
    <GridList
      aria-label="What to run"
      selectionMode="single"
      disallowEmptySelection
      selectedKeys={new Set([selectedKey])}
      onSelectionChange={onSelectionChange}
      className="overflow-hidden rounded-xl border border-app-border bg-app-bg-surface"
    >
      <GridListItem
        id={ENTIRE_WORKFLOW_KEY}
        textValue="Entire workflow"
        className="border-b border-app-border px-4 py-3.5 last:border-b-0"
      >
        {({ isSelected }) => (
          <ScopeRow
            isSelected={isSelected}
            title="Entire workflow"
            detail="All jobs from the workflow entrypoint"
          />
        )}
      </GridListItem>
      {jobs.map((job) => (
        <GridListItem
          key={job.id}
          id={job.id}
          textValue={job.id}
          className="border-b border-app-border px-4 py-3.5 last:border-b-0"
        >
          {({ isSelected }) => (
            <ScopeRow
              isSelected={isSelected}
              title={job.id}
              titleMono
              detail={
                job.acceptedExtensions.length > 0
                  ? `Single job · accepts ${job.acceptedExtensions
                      .map((extension) => `.${extension}`)
                      .join(", ")}`
                  : "Single job · accepts any file"
              }
            />
          )}
        </GridListItem>
      ))}
    </GridList>
  );
}

function ScopeRow({
  isSelected,
  title,
  detail,
  titleMono = false,
}: {
  isSelected: boolean;
  title: string;
  detail: string;
  titleMono?: boolean;
}) {
  return (
    <div className="flex items-start gap-3">
      <span
        aria-hidden
        className={[
          "mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full border motion-safe:transition-[border-color,background-color,box-shadow] motion-safe:duration-150 motion-safe:ease-out",
          isSelected
            ? "border-app-accent bg-app-accent"
            : "border-app-border bg-app-bg",
        ].join(" ")}
      >
        <span
          className={[
            "size-1.5 rounded-full bg-app-accent-foreground motion-safe:transition-transform motion-safe:duration-150 motion-safe:ease-out",
            isSelected ? "scale-100" : "scale-0",
          ].join(" ")}
        />
      </span>
      <div className="min-w-0">
        <Text className={["block font-medium", titleMono ? "font-mono" : ""].join(" ")}>
          {title}
        </Text>
        <Text className="mt-0.5 block" size="small" variant="muted">
          {detail}
        </Text>
      </div>
    </div>
  );
}

function FileDropZone({
  acceptLabel,
  inputPath,
  isDragging,
  dropTargetProps,
  onBrowse,
  onClear,
}: {
  acceptLabel: string;
  inputPath: string | null;
  isDragging: boolean;
  dropTargetProps: FileDropTargetProps;
  onBrowse: () => void;
  onClear: () => void;
}) {
  const fileName = inputPath
    ? (inputPath.split(/[/\\]/).filter(Boolean).pop() ?? inputPath)
    : null;

  return (
    <div
      {...dropTargetProps}
      className={[
        "flex min-h-56 flex-col items-center justify-center gap-3 rounded-xl border border-dashed px-6 py-10 text-center motion-safe:transition-[border-color,background-color] motion-safe:duration-150 motion-safe:ease-out",
        isDragging
          ? "border-app-accent bg-app-accent/5"
          : "border-app-border bg-app-bg-surface",
      ].join(" ")}
    >
      <Icon
        aria-hidden
        className="size-8 text-app-foreground-muted"
        definition={iconDefinitions.file}
      />
      {fileName ? (
        <>
          <Text className="font-medium">{fileName}</Text>
          <Text size="small" variant="muted">
            {inputPath}
          </Text>
          <div className="mt-2 flex items-center gap-2">
            <Button variant="secondary" onPress={onBrowse}>
              Choose another file
            </Button>
            <Button variant="ghost" onPress={onClear}>
              Clear
            </Button>
          </div>
        </>
      ) : (
        <>
          <Text className="font-medium">Drag and drop an input file</Text>
          <Text size="small" variant="muted">
            Accepted: {acceptLabel}
          </Text>
          <Button className="mt-2" variant="secondary" onPress={onBrowse}>
            Browse files
          </Button>
        </>
      )}
    </div>
  );
}

function commandErrorMessage(error: CommandError) {
  return error.message;
}
