# Zygo Python CLI protocol v0

This document describes how a runtime invokes the Zygo Python package and handles its output. [`schema.json`](schema.json) and its linked schemas define the payloads.

## Invocation

Run the CLI with a Python interpreter from the target environment:

```text
python -m zygo.cli.v0 --args JSON
```

Pass the complete command as one JSON command-line argument, not through stdin. The command selects the operation and includes its target and configuration.

The target is a module import path. Add an instance name, such as `package.module:workflow`, to select a specific instance. The module is required, but the CLI can discover the instance if its name is omitted.

The runtime chooses the interpreter, working directory, and environment. Importing the target runs its module's top-level Python code, so keep module initialization lightweight.

## Workflow metadata

`workflow_get_metadata` writes one UTF-8 result line to stdout with the prefix `ZYGO_IPC=`. This is a command result, not a side-effect publication. Importing the workflow may also produce ordinary output, so look for the prefix rather than treating all of stdout as JSON.

Metadata is synchronous and does not support HTTP delivery. The command exits with status 0 on success and a nonzero status on failure.

## Workflow execution and storage

`workflow_run_job` runs one job. The runtime supplies the workflow run and job run IDs. These IDs determine store paths, not which execution owns a publication.

When store configurations are omitted, the CLI looks for `pyproject.toml` from the workflow module's directory up to the working directory, inclusive. Without a configured store root, it uses a `zygo` directory beside the module. The default roots are `BASE/jobs/`, `BASE/workflows/`, and `BASE/cache/`. Explicit store configurations skip local discovery.

The job scope appends `wr=WORKFLOW_RUN_ID/jr=JOB_RUN_ID/` to its root. The workflow scope appends `wr=WORKFLOW_RUN_ID/`. The cache scope adds no run-specific path.

`ctx.store` uses the job scope by default. Use `ctx.store.scope("workflow")` or `ctx.store.scope("cache")` to select another scope. Scopes organize paths but do not restrict access.

The cache is shared by executions using the same root, not by all projects or users. Code must handle missing entries and be able to rebuild them. The cache does not provide automatic expiration or eviction.

Store backend options are passed to `fsspec.filesystem`. Each consumer enforces its own read and write requirements.

## Model training

`model_train` trains the target model. The model validates its training parameters and uses its declared defaults when parameters are omitted. Dataset loading enforces its format and read requirements.

The runtime allocates an output store for each training attempt. Artifact publications report completed writes, not successful training or a complete model artifact set. After successful training, the artifact set must remain unchanged so later loads are reproducible.


An inference command is not yet defined. Model loading happens inside the process that uses the model, not through a separate CLI command.

## Completion

Workflow execution and model training have no separate result payload. The CLI reports completion through its exit status and does not emit started, succeeded, failed, or cancelled events.

The runtime marks an attempt running after a successful process launch. It determines the final state from process completion, publication acceptance, and cancellation. Exit status 0 is not enough if required publications did not reach the runtime. Uncaught errors and publication failures cause a nonzero exit. Error details go to stderr.

## Publication delivery

### Stdout

When HTTP configuration is omitted or null, each publication is a flushed UTF-8 line in the form `ZYGO_IPC=JSON\n`. Ordinary program output may appear between these lines.

The runtime identifies the owning execution through the invoking process. It must read and accept all publications before declaring success. A broken stdout pipe is a publication failure.

Setting `PYTHONUNBUFFERED=1` can also help ordinary Python output appear promptly when stdout is piped.

### HTTP

With HTTP configuration, the CLI sends each emit call as a UTF-8 JSON `POST` with `Content-Type: application/json`. A request may contain several publications.

The runtime uses the callback URL or configured headers to identify and authorize the owning attempt. The CLI forwards headers on every request and retry without interpreting execution identity. Each new execution attempt needs its own binding. A header value alone does not prove authorization.

Any 2xx response means the receiver accepted the publication. Receivers must respond only after storing it durably. HTTP errors and network failures are retried. Each attempt gets its own timeout, and retries use a fixed delay. The retry count excludes the first attempt. When retries are exhausted, the CLI exits nonzero and the runtime marks the attempt failed.

A request may arrive even if the client sees a timeout or other failure. Retries reuse the same publication ID and body. Receivers must use the ID to avoid applying a publication twice. Delivery is not exactly-once.

Receivers must also handle late publications from cancelled or superseded attempts and check whether each event is allowed for its owning execution.

### Tags and secrets

A tag without an artifact reference applies to the owning execution, regardless of transport.

Command-line arguments may be visible in process listings. Putting credentials in JSON does not hide them.
