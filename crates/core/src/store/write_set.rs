use serde_json::Value;

use super::provider_interface::StorageProvider;
use super::{Store, StoreKey};

pub trait WriteSetReservation: Send {
    fn add_writes(&self, entries: &mut Vec<(StoreKey, Value)>);
    fn commit(&mut self);
}

/// A batch of writes that may be tied to a reservation.
///
/// Dropping an uncommitted write set also drops its reservation, allowing the
/// reservation to roll back any in-memory state it guarded.
pub struct WriteSet {
    entries: Vec<(StoreKey, Value)>,
    reservation: Option<Box<dyn WriteSetReservation>>,
}

impl Default for WriteSet {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteSet {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            reservation: None,
        }
    }

    pub fn push(&mut self, key: StoreKey, value: Value) {
        self.entries.push((key, value));
    }

    pub fn with_reservation<R>(mut self, reservation: R) -> Self
    where
        R: WriteSetReservation + 'static,
    {
        assert!(
            self.reservation.is_none(),
            "write set already has a reservation"
        );
        reservation.add_writes(&mut self.entries);
        self.reservation = Some(Box::new(reservation));
        self
    }

    pub async fn commit<S: StorageProvider>(mut self, store: &Store<S>) -> anyhow::Result<()> {
        store.put(&self.entries).await?;

        if let Some(reservation) = &mut self.reservation {
            reservation.commit();
        }

        Ok(())
    }
}

impl FromIterator<(StoreKey, Value)> for WriteSet {
    fn from_iter<T: IntoIterator<Item = (StoreKey, Value)>>(iter: T) -> Self {
        Self {
            entries: iter.into_iter().collect(),
            reservation: None,
        }
    }
}
