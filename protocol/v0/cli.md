# Zygo Python CLI protocol v0

This document specifies how an orchestrator invokes the Zygo Python package ([`schema.json`](schema.json) defines the payloads).
 
It is up to the orchestrator to implement the runtime environment.

## Executable and target

Invoke the module with a Python interpreter in that environment:

```text
python -m zygo.cli.v0 metadata TARGET
python -m zygo.cli.v0 run TARGET --args JSON [RUN_OPTIONS]
```

`TARGET` identifies a workflow by import path. `package.module:attribute` selects a `Workflow` instance. 

A module must be specified, but an instance may be discovered if ommitted.

It should be noted that importing a workflow executes its module's top-level Python code. Encourage light-weight module initialization.

The orchestrator is responsible for choosing the executable, working directory, and overall runtime environment.

## `metadata` command

The `metadata` command emits one prefixed UTF-8 metadata line on stdout:

```text
ZYGO_IPC={"id":"example","content_hash":"...","input_channel_id":"input","output_channel_id":"output","jobs":[],"channels":[]}
```

The part after `ZYGO_IPC=` is one `WorkflowMetadata` JSON object defined in [`schema.json`](schema.json). 

Other output from importing the workflow may also appear on stdout. Consumers should locate the prefixed metadata line rather than assume stdout contains only JSON. 

The command exits with status 0 on success, nonzero on failure. 

There are no HTTP options for metadata as the expectation is that this is always a synchronous operation.

## `run` command

This command runs a single job. The job run emits IPC messages to stdout (or via HTTP if `--http-config` is supplied).

`--args JSON` is required. Its value is a single JSON object matching `JobRunArgs` in [`schema.json`](schema.json), passed as **one command-line argument**, not on stdin:

```sh
python -m zygo.cli.v0 run myproject.main:workflow \
  --args '{"job_id":"my_job","data_reference_uri":"file:///input.txt","workflow_run_id":"wr-1","job_run_id":"jr-1"}'
```

`job_id` selects a job in the target workflow and `data_reference_uri` defines the input data for that job. The run IDs supply context to the Python job and store. The orchestrator is responsible for providing these IDs. 

A successful invocation exits with status 0 while uncaught/transport errors result in a nonzero exit. The process exit status indicates job completion while emitted messages indicate what happened during execution.

### IPC transports

#### stdout (default)

Without `--http-config`, each publication is a flushed UTF-8 stdout line of the form `ZYGO_IPC=JSON\n`, where `JSON` matches `IpcMessage` in [`schema.json`](schema.json). For example:

```text
ZYGO_IPC={"type":"channel_item_inserted","channel_id":"output","data_reference":"file:///result.txt"}
```

Workflow code may of course write ordinary output.

NB: Setting `PYTHONUNBUFFERED=1` can help ordinary workflow output appear promptly when stdout is piped.


#### HTTP messages

With `--http-config JSON`, the CLI sends **each** publication as a separate UTF-8 JSON `POST` to the configured `url`, with `Content-Type: application/json`. The body is `HttpIPCMessage` in [`schema.json`](schema.json), wrapping the stdout `IpcMessage` without its prefix:

```json
{"id":"<unique-message-id>","workflow_run_id":"wr-1","job_run_id":"jr-1","message":{"type":"channel_item_inserted","channel_id":"output","data_reference":"file:///result.txt"}}
```

The run IDs come from `--args` and correlate the publication with its workflow and job run. The CLI generates a unique `id` for each publication and sends the **same ID and body** on every retry of that publication. Receivers can deduplicate by `id` (and should retain that key for as long as duplicate delivery is possible); identical message contents from distinct publications have distinct IDs. The orchestrator supplies the receiver URL; this version does not prescribe an endpoint path or response body.

`--http-config` takes one JSON object matching `HttpConfig` in [`schema.json`](schema.json), as **one command-line argument**. Omit it for stdout delivery:

```sh
python -m zygo.cli.v0 run myproject.main:workflow \
  --args '{"job_id":"my_job","data_reference_uri":"file:///input.txt","workflow_run_id":"wr-1","job_run_id":"jr-1"}' \
  --http-config '{"url":"https://receiver.example/events","headers":{"Authorization":"Bearer TOKEN"},"timeout":30,"max_retries":3,"retry_interval":1}'
```

| Field | Meaning |
| --- | --- |
| `url` | Required full POST URL, beginning with `http://` or `https://`. |
| `headers` | Optional object mapping nonempty header names to string values; default `{}`. Names and values are trimmed. |
| `timeout` | Positive, finite number of seconds to wait for each HTTP response; default `30.0`. Each retry gets its own timeout. |
| `max_retries` | Nonnegative integer count of retries **after** the first failed attempt; default `3`. |
| `retry_interval` | Positive, finite base delay in seconds; default `5.0`. Retry delays are `retry_interval`, `2 × retry_interval`, `3 × retry_interval`, and so on. |

Unknown fields and invalid values are rejected before the job starts. Like any command-line argument, `headers` may be visible in process listings; do not assume putting secrets in JSON hides them.

Any 2xx response is treated as success. An HTTP error or network failure is retried until attempts are exhausted, after which the job will be failed. 

**Idempotency**: A request may have been received even if the client observes a failure (including a timeout). Receivers should use `HttpIPCMessage.id` to handle repeat deliveries without applying the same publication twice; delivery is not exactly-once. 


## Versioning boundary

The module path `zygo.cli.v0` selects this CLI version. `schema.json` defines payloads, not engine-internal event records or an HTTP service implementation. Changes to command syntax, framing or payload compatibility require a separately versioned interface.
