# Local Zygo Architecture

The local zygo crate exists to build local-first applications that run on a user's machine. We currently support two: The Desktop App and CLI.

## Multiprocess DB access

Because the Desktop App and CLI run in separate processes, they need to share access to the database. The alternative would be a daemon process and inter process communication, but I didn't want to build another *thing* just to share access to the database.

So, we use the experimental [turso multi-process access](https://docs.turso.tech/sql-reference/multiprocess-access) feature to share access to the database. This lets us open a connection in the Desktop App and CLI at the same time.

### Concurrent Writes Error Handling

Only one process can write at a time. 

A mutative operation like `INSERT`, `UPDATE`, or `DELETE` automatically acquires the write lock. If another process already holds it, the operation waits for the configured busy timeout. If the lock is not available within the busy timeout, the operation returns `SQLITE_BUSY`.

The configured busy timeout is the sole retry/wait policy for database writes. Base DB access and repositories do not retry `SQLITE_BUSY`; the error is returned directly when the timeout expires. Transactions can use `BEGIN IMMEDIATE` to indicate they want the write lock before starting the transaction.

### Drawbacks

Obviously, it's an experimental feature and they state that they may change the api and storage format so a migration path may need to be built in the future.

Another downside is that Windows support is not available. So until Turso adds Windows support, Windows users can only use one local app at a time.


## DB Structure

The DB has two responsibilities:
1. Have a durable record of all workflow runs.
2. Summarize/index the run data for application features.

To this end we use "summary" tables that store aggregated data computed over each runs event stream. These are designed to only require the event stream to recompute and nothing else.

### Tables
- `workflow_run`: Durable record of a local workflow run.
- `workflow_run_summary`: Top-level summary of workflow run data, e.g. status, total duration, # of active jobs, etc.
- `job_run_summary`: Top-level summary of job run data, e.g. status, duration, # of retries, etc.
- `tags`: Normalized table of all tags associated with workflow runs, jobs, and data references.
- `tag_associations`: Store the many-to-many relationships between tags and other tables along with the tag instance's value.
