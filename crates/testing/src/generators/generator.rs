//! Deterministic generators for the entities we want to exercise under test.
//!
//! This module follows the TigerStyle approach to deterministic simulation
//! testing (DST): instead of hand-writing a handful of fixtures, we describe the
//! *space* of valid inputs and draw concrete samples from a single seeded RNG.
//! Replaying the same seed reproduces the exact same world, which makes any bug
//! we find trivially reproducible (and shrinkable).
//!
//! # Shape of the module
//!
//! Each domain entity we generate lives in its own flat module and exposes a
//! "generator" struct that captures *how* that entity is produced (the
//! distributions, pools, and bounds). The generators compose bottom-up and emit
//! *blueprints* — id-free, name-based descriptions shaped like the requests the
//! orchestrator accepts. The orchestrator is the source of truth for ids; it
//! mints them at registration and returns them, so generators never invent ids:
//!
//! ```text
//! world           (the whole simulated run: a WorldBlueprint)
//! ├── workflow     (the graph: a WorkflowBlueprint)
//! │   ├── job      (units of work, by name + wiring)
//! │   │   └── entrypoint (how a job is executed: local vs remote)
//! └── event        (external stimuli fed into the run)
//! ```
//!
//! # Composing into an end-to-end test
//!
//! ```no_run
//! use testing::generators::{world::{World, WorldGenerator}, GenerateExt};
//!
//! // One seed -> one fully-formed blueprint, then accumulate the real ids the
//! // orchestrator returns as we drive it.
//! let blueprint = WorldGenerator::default().generate_seeded(42);
//! let mut world = World::new(blueprint);
//! let request = world.register_request();
//! // ... send `request`, then `world.apply_registration(response)`, then feed
//! // `world.input_event_requests()`, build `world.run_scope()`, step, assert.
//! ```

use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use uuid::Uuid;

pub mod entrypoint;
pub mod event;
pub mod job;
pub mod workflow;
pub mod world;

/// A deterministic generator for a single domain value.
///
/// The implementing struct describes *the ways a value can be generated*; a call
/// to [`Generate::generate`] draws one concrete sample using the supplied RNG.
/// Threading the same [`ChaCha8Rng`] through every generator is what keeps an
/// entire world reproducible from one seed.
///
/// [`Context`](Generate::Context) carries any information a generator needs from
/// its parent (for example, the shared `WorkflowVersionId` that all jobs in a
/// schema belong to). Generators that need nothing from their parent set
/// `Context = ()` and gain the ergonomic helpers on [`GenerateExt`].
pub trait Generate {
    /// The domain value produced by this generator.
    type Output;
    /// Information supplied by the parent generator at draw time.
    type Context;

    /// Draw a single value, advancing `rng`.
    fn generate(&self, rng: &mut ChaCha8Rng, context: Self::Context) -> Self::Output;
}

/// Ergonomic entry points for context-free generators (`Context = ()`).
///
/// This is where the "easy to use" surface lives: most callers only ever need
/// [`generate_seeded`](GenerateExt::generate_seeded) to turn a `u64` seed into a
/// fully realized value.
pub trait GenerateExt: Generate<Context = ()> {
    /// Draw a value from an existing RNG without threading a unit context.
    fn generate_value(&self, rng: &mut ChaCha8Rng) -> Self::Output {
        self.generate(rng, ())
    }

    /// Draw a value from a fresh RNG seeded with `seed`.
    ///
    /// This is the canonical DST entry point: a seed in, a deterministic value
    /// out.
    fn generate_seeded(&self, seed: u64) -> Self::Output {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        self.generate(&mut rng, ())
    }
}

impl<T> GenerateExt for T where T: Generate<Context = ()> {}

/// Draw a realistic-looking UUID from `rng`.
///
/// The bytes are fully random (we do not bother stamping version/variant bits)
/// because these ids are only ever treated as opaque, non-empty identifiers.
pub(crate) fn random_uuid(rng: &mut ChaCha8Rng) -> Uuid {
    Uuid::from_u128(rng.random::<u128>())
}

/// Pick a uniformly random element from a non-empty slice.
pub(crate) fn choose<'a, T>(rng: &mut ChaCha8Rng, items: &'a [T]) -> &'a T {
    debug_assert!(!items.is_empty(), "cannot choose from an empty slice");
    &items[rng.random_range(0..items.len())]
}
