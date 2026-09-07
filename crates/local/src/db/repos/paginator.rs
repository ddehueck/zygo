use crate::DbResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub id: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Page<T> {
    pub next: Option<Cursor>, // No cursor if this is the last page
    pub data: Vec<T>,
}

#[allow(async_fn_in_trait)]
pub trait CursorPaginator {
    type Item;

    async fn list(&self, cursor: Option<Cursor>, limit: i64) -> DbResult<Page<Self::Item>>;
}
