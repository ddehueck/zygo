//! Generation of external stimuli: the [`DataReference`]s that get fed into a
//! run's source channel to kick it off.
//!
//! Varying the number of inputs and their etags explores different fan-in and
//! idempotency paths (the job-run id is derived from the data reference's
//! uri + etag, so distinct etags produce distinct job runs).

use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use zygo_core::models::DataReference;

use crate::generators::Generate;

/// Describes how the set of input data references for a run is generated.
#[derive(Debug, Clone)]
pub struct EventGenerator {
    /// Minimum number of inputs to feed into the source channel.
    pub min_inputs: usize,
    /// Maximum number of inputs to feed into the source channel.
    pub max_inputs: usize,
}

impl Default for EventGenerator {
    fn default() -> Self {
        Self {
            min_inputs: 1,
            max_inputs: 3,
        }
    }
}

impl Generate for EventGenerator {
    type Output = Vec<DataReference>;
    type Context = ();

    fn generate(&self, rng: &mut ChaCha8Rng, _context: ()) -> Vec<DataReference> {
        let lower = self.min_inputs.max(1);
        let upper = self.max_inputs.max(lower);
        let count = rng.random_range(lower..=upper);

        (0..count)
            .map(|index| {
                // A random etag per input keeps the derived job-run ids distinct
                // while remaining reproducible from the seed.
                let etag_nonce = rng.random::<u64>();
                DataReference {
                    uri: format!("dst://input/{index}"),
                    etag: format!("dst-etag-{etag_nonce:016x}"),
                    content_type: Some("application/octet-stream".to_owned()),
                    size_bytes: Some(rng.random_range(0..=4096u64)),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::GenerateExt;

    #[test]
    fn respects_input_bounds() {
        let generator = EventGenerator {
            min_inputs: 2,
            max_inputs: 5,
        };

        for seed in 0..64 {
            let inputs = generator.generate_seeded(seed);
            assert!((2..=5).contains(&inputs.len()));
        }
    }
}
