# Local Sync Architecture

As other threads run a workflow local applications should be kept up to date. This way a user can run a workflow via the cli
and instantly see the run start on in the desktop app as well.

## Overview

A client will subscribe to a tauri channel holds a connection to the rust backend. Through this connection they can receive batches of delta objects that represent changes to the local database. The react app can then apply those changes to it's local tanstack db collection.

## Normalized Collections

All clients will mirror the same normalized collections we have in the local db file. Then using tools like TanstackDB the client can decide the particular projection of data it needs to best serve it's usecase. This creates a clear separation between the client's unique data needs and the local crate's core concerns.

Right now we have:
- Workflow Run
- Job Run
- Tag
- Date References
- Logs

## Implementation

Because zygo can run on multiple processes, we need to record changes at the db level so that all processes can see the same changes. With these changes, we can then tell connected clients the delta of what has changed relative to their local state.

### Capturing DB Changes

Sync is built on top of [turso's change data capture (CDC) feature](https://docs.turso.tech/tursodb/cdc). Conceptually, for every change to every row in every table, a record of that change is ordered and stored in the

| Column | Type | Description |
| --- | --- | --- |
| `change_id` | `INTEGER` | Auto-incrementing unique identifier |
| `change_time` | `INTEGER` | Timestamp (Unix epoch) |
| `change_txn_id` | `INTEGER` | Transaction ID (groups rows into transactions) |
| `change_type` | `INTEGER` | 1 = INSERT, 0 = UPDATE, -1 = DELETE, 2 = COMMIT |
| `table_name` | `TEXT` | Name of the table that was changed |
| `id` | varies | Primary key/rowid of the changed row |
| `before` | `BLOB` | Row data before the change (modes: before, full) |
| `after` | `BLOB` | Row data after the change (modes: after, full) |
| `updates` | `BLOB` | Per-column change details (mode: full) |

This takes care of the core concern of what changed and how.

**A note on CDC, persisted models, and raw rows**: The CDC repository returns raw `CdcRow` records, including their historical JSON after-values which match to a `*SqlRow` type. These are then converted into Rust-native `*Model` values. Normal repository queries use the same Row-to-Model conversions (for example, integer row values becomes a boolean as needed).

### Client Sync Protocol

When a client connects:
1. They immediately start receiving change notifications from the server.
2. They then receive the full snapshot of the collection(s) they are interested in.
3. They apply the changes they receive to their local database.

Crucially, they start receiving change notifications first to avoid a race where they miss changes that occur while they are receiving the full snapshot. The tradeoff of this approach is that all notifications have to result in idempotent updates to the local database as the updates may already be reflected in the snapshot.

#### Local Update Operations

These are the idempotent operations that are applied to the local database when a change notification is received.
- `Upsert`: Inserts or updates a row in the local database.
- `Delete`: Deletes a row from the local database.
- `Resync`: Clear the local database, resubscribe to the server, and receive a full snapshot.

### The Broker

The broker is responsible for managing the sync connection with the local crate and local database and the client connection. 
It's responsibilities include:
- Polling for relevant changes in the CDC table
- Managing/incrementing the high-water mark for the client
- Determining if the client needs to resync when certain changes occur - e.g. schema changes

#### Pacing 

Because an active workflow run results in many changes in a short span of time, we don't want to overwhelm clients with a barrage of change notifications. Instead, the broker that manages the sync connection limits the rate at which change notifications are sent.


### The Desktop App

As the primary target client, we discuss the implementation specific with tauri and this core local sync module. The initial related-data snapshot is cached by TanStack Query without automatic staleness or garbage collection, then projected into local-only TanStack DB collections. CDC deltas update those in-memory collections directly.

The desktop app is built on Tauri which has a JS layer and Rust backend. The JS layer drives the UI and can communicate with the Rust backend via Tauri's IPC functionality. At a high level, the desktop app's Rust backend will have a background thread that runs the broker which will then emit a "poke" event to the JS layer when changes are detected. When the JS layer receives the "poke" event, it call a command to ingest the changes into the JS layer's TanstackDB instance. Once the command is completed the JS layer will inform the Broker that the changes have been ingested and the Broker will increment the high-water mark for the client.

#### Data Size

The desktop app will not hold the full contents of the the local crate' database. Instead it will fetch a limited snapshot. As an example, imagine a large database of workflow runs:

- The desktop app will fetch the first 100 workflow runs together with their related job runs and tags, then store each entity in its normalized TanStack DB collection.
- If a change notification is received for an out of set run, the desktop app will ignore it. (This only applies to the delete operation, upserts are always applied)
- If the desktop app wants to fetch the next 100 workflow runs, it will begin to buffer all change notifications, fetch the snapshot, the apply the buffered changes and beginning listening for changes again.
