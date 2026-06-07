use crate::engine::RunScope;
use crate::models::{Event, StreamItem};
use crate::store::{StorageProvider, Store};
use crate::stream::Stream;

pub async fn handle_event<S: StorageProvider + 'static>(
    store: &Store<S>,
    scope: RunScope,
    event: Event,
) -> anyhow::Result<()> {
    let stream = Stream::new(store.clone(), scope.clone());
    let write_set = stream.append(vec![StreamItem::Event(event)]).await?;
    store.commit_write_set(write_set).await?;
    Ok(())
}
