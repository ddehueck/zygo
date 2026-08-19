# zygo

`zygo` is the Python authoring and job-execution side of Zygo. It lets users
define workflows in Python and provides the process boundary through which the
Rust runners inspect workflows and execute jobs.

## Role in the architecture

The package has two responsibilities:

- provide the `Workflow` APIs used to author a workflow;
- provide a cli for the Rust runners to inspect workflows and execute jobs;

```text
Python workflow
      |
      | metadata
      v
Rust CLI -> zygo-core
               |
               | run one job
               v
        Python job process
               |
               | published references
               v
          zygo-core stream
```

The Python `Store` reads and writes workflow payloads through `fsspec`. It is
not the same store as Rust's `StorageProvider`, which persists orchestration
state such as streams, snapshots, and cache records. The two sides communicate
using `DataReference` values.

## Local development

```bash
just setup
```
