use std::env;
use std::time::{Duration, Instant};

use rand::RngExt;
use testing::generators::GenerateExt;
use testing::generators::workflow::{Topology, WorkflowGenerator};
use testing::generators::world::WorldGenerator;
use testing::invariants;
use tracing::{info, warn};
use zygo_core::store::{MemoryStore, Store};
use zygo_core::{WorkflowRunReader, Zygo, ZygoConfig};

/// Maximum time to wait for the actor-driven workflow run to become terminal.
const RUN_TIMEOUT: Duration = Duration::from_secs(10);
const STATUS_POLL_INTERVAL: Duration = Duration::from_millis(25);

fn main() {
    tracing_subscriber::fmt::init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime")
        .block_on(run())
        .expect("testing run failed");
}

async fn run() -> anyhow::Result<()> {
    let seed = env::args()
        .nth(1)
        .map(|value| value.parse::<u64>())
        .transpose()?
        .unwrap_or_else(|| rand::rng().random_range(0..u64::MAX));

    info!(seed, "generating test world");

    // A single job exercises the complete service/actor/worker lifecycle without
    // requiring a job to publish downstream channel data.
    let generator = WorldGenerator {
        workflow: WorkflowGenerator {
            topologies: vec![Topology::Linear],
            max_chain_length: 1,
            ..WorkflowGenerator::default()
        },
        ..WorldGenerator::default()
    };
    let world = generator.generate_seeded(seed);
    info!(world = ?world, "generated test world");

    // Zygo currently accepts one input per run. Preserve the previous harness
    // behavior, which generated several candidates but submitted only the first.
    let input = world
        .inputs
        .into_iter()
        .next()
        .expect("world generator must produce at least one input");

    let store = Store::new(MemoryStore::new());
    let zygo = Zygo::new(store.clone(), ZygoConfig::new(1));
    let run_id = zygo.run(input, world.schema).await?;
    info!(%run_id, "submitted workflow run");

    let reader = WorkflowRunReader::new(store, run_id);
    let hit_timeout = wait_for_terminal(&reader).await?;
    let records = reader.stream().collect().await?;

    let invariants: Vec<Box<dyn invariants::Invariant>> = vec![
        Box::new(invariants::CheckTerminalStatus::new(hit_timeout)),
        Box::new(invariants::CheckOrderedRunEvents::new(records)),
    ];

    let runner = invariants::InvariantRunner::default();
    for invariant in &invariants {
        runner.run(invariant.as_ref());
    }

    Ok(())
}

async fn wait_for_terminal(reader: &WorkflowRunReader<MemoryStore>) -> anyhow::Result<bool> {
    let started_at = Instant::now();

    loop {
        if let Some(status) = reader.status().await? {
            if status.is_terminal() {
                info!(?status, run_id = %reader.run_id(), "workflow run reached terminal status");
                return Ok(false);
            }
        }

        if started_at.elapsed() >= RUN_TIMEOUT {
            warn!(
                run_id = %reader.run_id(),
                timeout = ?RUN_TIMEOUT,
                "workflow run did not reach a terminal status before timeout"
            );
            return Ok(true);
        }

        tokio::time::sleep(STATUS_POLL_INTERVAL).await;
    }
}
