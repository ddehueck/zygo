use crate::models::SequenceId;

pub struct StreamCursor {
    pub next_id: SequenceId,
}

impl StreamCursor {
    pub fn new(next_id: SequenceId) -> Self {
        Self { next_id }
    }

    pub fn next(self) -> Self {
        Self {
            next_id: self.next_id.next(),
        }
    }
}
