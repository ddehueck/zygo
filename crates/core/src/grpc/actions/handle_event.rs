use crate::models::{Event, StreamItem};
use crate::store::{StorageProvider, Store};
use crate::stream::Stream;

pub async fn handle_event<S: StorageProvider>(
    store: &Store<S>,
    event: Event,
) -> anyhow::Result<()> {
    let stream = Stream::for_run(
        store.clone(),
        event.workflow_id.clone(),
        event.workflow_version_id.clone(),
        event.workflow_run_id.clone(),
    );
    let write_set = stream.append(vec![StreamItem::Event(event)]).await?;
    store.commit_write_set(write_set).await?;
    Ok(())
}
