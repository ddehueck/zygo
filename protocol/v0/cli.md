# Zygo Python CLI protocol v0

This document specifies how a runtime invokes the Zygo Python package ([`schema.json`](schema.json) defines the payloads).

## Executable and target

Invoke the user-defined module through the unified versioned CLI with a Python interpreter in that environment. Workflow commands are implemented:

```text
python -m zygo.cli.v0 workflow metadata TARGET
python -m zygo.cli.v0 workflow run TARGET --args JSON [--store-config JSON] [RUN_OPTIONS]
```

The unified `ml train` command parses the training contract and dispatches to a stub training executor that intentionally raises `NotImplementedError`. Its invocation is:

```text
python -m zygo.cli.v0 ml train TARGET --dataset-config JSON --store-config JSON [RUN_OPTIONS]
```

`TARGET` identifies a zygo module by import path. `package.module:workflow` selects a `Workflow` instance named `workflow`.

A module must be specified, but an instance may be discovered if omitted.

It should be noted that importing a workflow executes its module's top-level Python code. Encourage light-weight module initialization.

The runtime is responsible for choosing the executable, working directory, and overall environment.

# Workflow Commands

## `workflow metadata` command

The `workflow metadata` command emits one prefixed UTF-8 metadata line on stdout:

```text
ZYGO_IPC={"id":"example","content_hash":"...","input_channel_id":"input","output_channel_id":"output","jobs":[],"channels":[]}
```

The part after `ZYGO_IPC=` is one `WorkflowMetadata` JSON object defined in [`schema.json`](schema.json). 

Other output from importing the workflow may also appear on stdout. Consumers should locate the prefixed metadata line rather than assume stdout contains only JSON. 

The command exits with status 0 on success, nonzero on failure. 

There are no HTTP options for metadata as the expectation is that this is always a synchronous operation.

## `workflow run` command

This command runs a single job. The job run emits IPC messages to stdout (or via HTTP if `--http-config` is supplied).

`--args JSON` is required. Its value is a single JSON object matching `JobRunArgs` in [`schema.json`](schema.json), passed as **one command-line argument**, not on stdin:

```sh
python -m zygo.cli.v0 workflow run myproject.main:workflow \
  --args '{"job_id":"my_job","data_reference_uri":"file:///input.txt","workflow_run_id":"wr-1","job_run_id":"jr-1"}'
```

`job_id` selects a job in the target workflow and `data_reference_uri` defines the input data for that job. The run IDs supply context to the Python job and store. The orchestrator is responsible for providing these IDs.

`--store-config JSON` is optional and takes one `WorkflowStoreConfig` JSON object as one command-line argument, separate from the job args. All three fields, `job`, `workflow`, and `cache`, are required when supplied. Each is a `StoreConfig` with a `root_uri` and optional `kwargs` passed as keyword arguments to `fsspec.filesystem`. Backend options map string keys to arbitrary JSON values, including numbers, booleans, null, arrays, and nested objects. Omitted `kwargs` defaults to `{}`. Nonfinite numbers are rejected.

```sh
python -m zygo.cli.v0 workflow run myproject.main:workflow \
  --args '{"job_id":"my_job","data_reference_uri":"file:///input.txt","workflow_run_id":"wr-1","job_run_id":"jr-1"}' \
  --store-config '{"job":{"root_uri":"file:///tmp/my-results/jobs/","kwargs":{"auto_mkdir":true}},"workflow":{"root_uri":"file:///tmp/my-results/workflows/"},"cache":{"root_uri":"file:///tmp/my-results/cache/"}}'
```

When `--store-config` is omitted, the CLI looks for `pyproject.toml` from the imported workflow module's directory up to the working directory, inclusive. Without a configured store root, the base defaults to a `zygo` directory beside the module. Default scope roots are `BASE/jobs/`, `BASE/workflows/`, and `BASE/cache/`. Explicit configurations bypass local default discovery entirely.

Scope configs supply base roots. The job scope appends `wr=WORKFLOW_RUN_ID/jr=JOB_RUN_ID/`, the workflow scope appends `wr=WORKFLOW_RUN_ID/`, and the cache scope adds no run partitions. `ctx.store` defaults to job scope, with `ctx.store.scope("workflow")` and `ctx.store.scope("cache")` selecting the others. These scopes are namespace conventions, not access-control boundaries. The cache is shared across executions using the same configured root, not across all projects or users. Callers must tolerate missing entries and be able to reconstruct them. The name does not imply automatic expiration or eviction.

A successful invocation exits with status 0 while uncaught/transport errors result in a nonzero exit. The process exit status indicates job completion while emitted messages indicate what happened during execution.

The runtime owns execution lifecycle. The CLI does not emit started, succeeded, failed, or cancelled events. The runtime marks an attempt running after successful process launch and determines its terminal state from process completion, publication acceptance, and cancellation. Exit 0 is not sufficient if required publications failed to reach the runtime. Error details remain available in stderr.

For stdout delivery, the runtime must drain and accept the process's publications before declaring success. Publication errors, including a broken stdout pipe, cause a nonzero CLI exit.

# Model Commands

## `ml train` command (stub executor)

The CLI target selects the model. `--dataset-config JSON` and `--store-config JSON` are both required and passed as separate command-line arguments. There is no `--args` option for training.

`--dataset-config` accepts a `DatasetConfig` with a nonempty `uri` and optional backend `kwargs`. `--store-config` accepts a single `StoreConfig` with a `root_uri` and optional backend `kwargs`, not a `WorkflowStoreConfig`. Both option dictionaries default to `{}` and accept arbitrary finite JSON values.

```sh
python -m zygo.cli.v0 ml train myproject.models:classifier \
  --dataset-config '{"uri":"file:///datasets/training/","kwargs":{}}' \
  --store-config '{"root_uri":"file:///models/classifier/attempt-123/","kwargs":{"auto_mkdir":true}}'
```

The runtime allocates a model store dedicated to the attempt. Artifact publications report completed writes, not successful training. The runtime determines training success from process completion and publication acceptance. Successfully trained artifact sets must remain unchanged for reproducible loading. The CLI parses this contract, but the stub training executor intentionally raises `NotImplementedError`, resulting in a nonzero exit. Actual training is left for downstream implementation. The CLI emits no lifecycle events.

## Infer Command

An inference command's arguments are not yet defined. Loading is an internal step of the process that uses the live model, not a standalone CLI operation in this protocol.


# IPC transports

#### stdout (default)

Without `--http-config`, each publication is a flushed UTF-8 stdout line of the form `ZYGO_IPC=JSON\n`, where `JSON` matches `IpcMessage` in [`schema.json`](schema.json). For example:

```text
ZYGO_IPC={"type":"data_reference_inserted","data_reference":"file:///result.txt"}
ZYGO_IPC={"type":"channel_item_inserted","channel_id":"output","data_reference":"file:///result.txt"}
ZYGO_IPC={"type":"tag_inserted","value":"checkpoint","data_reference":"file:///result.txt"}
```

The runtime attributes these events through the invoking process. A tag with an absent or null `data_reference` applies to that execution. Workflow code may of course write ordinary output.

NB: Setting `PYTHONUNBUFFERED=1` can help ordinary workflow output appear promptly when stdout is piped.


#### HTTP messages

With `--http-config JSON`, the CLI sends each emit call as a UTF-8 JSON `POST` to the configured `url`, with `Content-Type: application/json`. One POST may include one or more IPC messages.

The body is `HttpIPCMessage` in [`schema.json`](schema.json), wrapping a batch of stdout `IpcMessage` values without their prefixes:

```json
{
  "id": "unique-message-id",
  "messages": [
    {
      "type": "channel_item_inserted",
      "channel_id": "output",
      "data_reference": "file:///result.txt"
    }
  ]
}
```

The body contains no execution context. The runtime binds the configured URL or headers to a specific execution attempt and uses that binding to attribute and authorize publications. An opaque attempt handle can identify the runtime's execution record without sending the full context. The protocol does not prescribe a header name. Configured headers are forwarded on every publication and retry.

`--http-config` takes one JSON object matching `HttpConfig` in [`schema.json`](schema.json), as **one command-line argument**. Omit it for stdout delivery:

```sh
python -m zygo.cli.v0 workflow run myproject.main:workflow \
  --args '{"job_id":"my_job","data_reference_uri":"file:///input.txt","workflow_run_id":"wr-1","job_run_id":"jr-1"}' \
  --http-config '{"url":"https://receiver.example/events","headers":{"Authorization":"Bearer TOKEN","X-Execution-Handle":"opaque-attempt-handle"},"timeout":30,"max_retries":3,"retry_interval":1}'
```

| Field | Meaning |
| --- | --- |
| `url` | Required full POST URL, beginning with `http://` or `https://`. |
| `headers` | Optional object mapping nonempty header names to string values. Defaults to `{}`. Names and values are trimmed. |
| `timeout` | Positive, finite number of seconds to wait for each HTTP response. Defaults to `30.0`. Each retry gets its own timeout. |
| `max_retries` | Nonnegative integer count of retries **after** the first failed attempt. Defaults to `3`. |
| `retry_interval` | Positive, finite base delay in seconds. Defaults to `5.0`. Retries are fixed i.e. `retry_interval` seconds between attempts. |

Unknown fields and invalid values are rejected before the job starts. Like any command-line argument, `headers` may be visible in process listings. Do not assume putting secrets in JSON hides them.

Any 2xx response is treated as publication acceptance. Receivers must acknowledge only after durable acceptance. An HTTP error or network failure is retried until attempts are exhausted, after which the CLI exits nonzero and the runtime marks the attempt failed.

**Idempotency**: A request may have been received even if the client observes a failure (including a timeout). Receivers should use `HttpIPCMessage.id` to handle repeat deliveries without applying the same publication twice. The ID and body are reused unchanged on retries. Delivery is not exactly-once.

Each retry of an execution needs a distinct attempt binding. Receivers must handle late publications from cancelled or superseded attempts and validate whether the event is permitted for its owning execution. A header value alone does not establish authorization.
