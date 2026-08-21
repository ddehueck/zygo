# Desktop App Architecture

The desktop app is built using GPUI for a native experience that can interact with `zygo-core` and `zygo-local` directly.

The key architectural concerns are:
- Data fetching
- UI rendering
- UI State management

### Project Structure

We will follow a nested feature-based structure for this dekstop app. Shared and global concerns will be handled in top-level files, while feature-specific concerns will be handled in a features directory.

e.g.
```
src/
├── main.rs
├── stores/
│   ├── workflow_run_store.rs
│   ├── ...
├── ui/
│   ├── list/  // shared list component
│   │   ├── list.rs
│   │   ├── list.stories.rs
│   │   └── list.css
│   ├── ...
├── features/
│   ├── runs/
│   │   ├── ui/
│   │   │   ├── run_list.rs
│   │   │   ├── run_list.stories.rs
│   │   │   └── run_list.css
│   │   ├── run.rs
│   │   └── ...
│   └── ...
```

## Data Fetching

Because GPUI allows us to write a reactive UI in Rust, we can call the `zygo-local` API directly from the UI layer. This makes data fetching efficient and easy to implement.

The `zygo-local` crate exposes Repositories that are then consumed in Data Stores.

### Data Stores

We create Stores for each data model (e.g. `WorkflowRunStore`, `TagStore`) to make data access easy and non-blocking for the UI.

These are Entities that hold state for GPUI.

These stores are responsible for fetching, caching, and providing data to the UI. They fetch the data from the `zygo-local` API in the background using by spawning a `cx.spawn` or `cx.background_spawn` thread through gpui's context object.

On top of the store Entity state that can be observed by a UI component, stores can also expose a simple API of events they emit. These events are subscribed to by the UI via. For example, 

```rust
impl Sidebar {
    fn new(
        store: Entity<ProjectStore>,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscription = cx.subscribe(
            &store,
            |sidebar, event, cx| {
                match event {
                    ProjectStoreEvent::ProjectCreated(id) => {
                        sidebar.highlighted_project = Some(*id);
                        cx.notify();
                    }
                    ProjectStoreEvent::ProjectDeleted(id) => {}
                    ProjectStoreEvent::RefreshCompleted { count } => {}
                }
            },
        );
        ...
    }
}
```

**A note on filtering**: we will simply start with filtering in memory only. As performance becomes an issue, we can design some kind of predicate pushdown to give the UI more granular data fetching abilities.


**GPUI Integration:** All stores are defined in a top level `stores` module and then exposed to the GPUI App via an injected stores Entity.

## UI Rendering

GPUI provides two traits to render UI: Render and RenderOnce. Render trait implementations are backed by an Entity with persistent GPUI state. We will use each as appropriate.

Because I write a lot of react, I will refer to structs that implement these traits as `components`. And because this architecture is focused on a clean seperation of concernts the analogy should hold.

## UI State Management

todo


## Misc Resources

Here are some useful resources for learning more about concepts discussed in this doc:

- https://github.com/astrimid/gpui-book2/blob/main/entity-view-foundation.md
