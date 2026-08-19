# zygo-core

`zygo-core` is Zygo's in-process orchestration runtime. It accepts a runtime
`WorkflowSchema` and an input `DataReference`, advances the resulting workflow
run, and persists its orchestration state through a given `StorageProvider`.

## Role in the architecture

Core owns the domain model and mechanics of orchestration:

- one active `Actor` coordinates each workflow run;
- the `Engine` processes an ordered stream of events and commands one at a time;
- workers execute jobs in capacity-limited subprocesses;
- snapshots and stream records make run state observable and replayable;
- cached job results can be replayed without executing the job again.

Core exposes a single `Zygo` service that allows consumers to start and subscribe to workflow runs.

```text
WorkflowSchema + DataReference
              |
              v
          Zygo service
              |
      Actor for each run
              |
    Event -> Engine -> Command
              |
       Worker or replay
              |
       persisted stream
```

Core does not define or discover workflows.

## Glossary

- **Workflow schema** — The runtime graph of jobs, channels, edges, entrypoints,
  and a designated input channel. It is already constructed when core receives
  it.
- **Workflow run** — One execution of a workflow schema for a particular input.
- **Job** — A definition within a workflow schema, including its identity,
  content hash, and executable entrypoint.
- **Job run** — One deterministic invocation of a job for an input reference.
- **Channel** — A named position in the workflow graph through which data
  references are published.
- **Data reference** — Metadata identifying payload data, such as its URI and
  content identity. Core orchestrates references, not the payload bytes.
- **Event** — A persisted fact about something that happened during a workflow
  run.
- **Command** — Persisted intent produced from an event and interpreted by the
  engine.
- **Run stream** — The ordered, per-run log containing both events and commands.
- **Actor** — The task that owns the concurrent lifecycle of one active workflow
  run. An actor is not a job worker.
- **Engine** — The state machine that advances a run through its persisted
  stream.
- **Worker** — Capacity for executing one job, currently as a local Python
  subprocess.
- **Snapshot** — Derived run state paired with the next unread stream position.
- **Replay** — Re-emitting the cached output events of a job run instead of
  executing that job again.
- **Storage provider** — The atomic key-value interface used for storing streams and snapshots.
