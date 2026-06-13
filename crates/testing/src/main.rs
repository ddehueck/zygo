use std::env;
use std::time::{Duration, Instant};

use rand::RngExt;
use testing::generators::GenerateExt;
use testing::generators::entrypoint::{
    DEFAULT_ORCHESTRATOR_ENDPOINT, ScriptGenerator, responder_script_path,
};
use testing::generators::world::{World, WorldGenerator};
use testing::invariants;
use zygo_core::store::MemoryStore;
use tonic::Request;
use tonic::transport::Channel;
use tracing::{error, info, warn};
use zygo_core::engine::{Engine, RunScope, StepResult};
use zygo_core::grpc::OrchestratorService;
use zygo_core::orchestrator_proto::orchestrator_service_client::OrchestratorServiceClient;
use zygo_core::store::{StorageProvider, Store};
use zygo_core::stream::{Stream, StreamReader};

/// How long the engine may sit idle (no new stream items) before we give up
/// waiting for jobs to report back. Without a cap a run whose jobs never respond
/// would spin forever.
const IDLE_TIMEOUT: Duration = Duration::from_secs(10);

// Each run:
// - Generates a random world from a seed (deterministic simulation testing).
// - Runs the resulting state through the engine.
// - Prints the final state.

fn main() {
    tracing_subscriber::fmt::init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime")
        .block_on(run());
}

async fn run() {
    // Resolve seed either from the first argument or generate one.
    let args: Vec<String> = env::args().collect();
    let seed = args
        .get(1)
        .map(|s| s.parse::<u64>().unwrap())
        .unwrap_or(rand::rng().random_range(0..u64::MAX));

    info!("Using seed: {}", seed);

    // Generate a random world blueprint given the seed. The blueprint is pure
    // intent: it has no orchestrator-minted ids yet.
    let blueprint = WorldGenerator::default().generate_seeded(seed);
    info!("Generated world blueprint: {:?}", blueprint);

    // Wrap the blueprint so we can accumulate the real ids the orchestrator
    // returns as we drive it.
    let mut world = World::new(blueprint);

    // Materialize the responder script that every local job runs as its
    // entrypoint. When the engine runs a local job it shells out to this script,
    // which reports the job's lifecycle (JobStarted, then JobSucceeded) back to
    // this very orchestrator over gRPC — the "response event" that drives the
    // run forward.
    let script_path = responder_script_path();
    std::fs::write(&script_path, ScriptGenerator::default().render())
        .expect("failed to install responder script");
    info!("Installed responder script at {}", script_path.display());

    // Create a store and start the orchestrator gRPC service in the same mode
    // the world was generated for (all jobs are local or all remote).
    let store = Store::new(MemoryStore::new());
    let orchestrator_service = OrchestratorService::new(store.clone(), world.mode());
    let addr = DEFAULT_ORCHESTRATOR_ENDPOINT.parse().unwrap();
    let router = orchestrator_service
        .into_router()
        .expect("failed to build orchestrator router");

    // Start the orchestrator server in a separate task.
    tokio::spawn(async move {
        router
            .serve(addr)
            .await
            .expect("orchestrator server failed");
    });

    // Connect a client to drive the orchestrator over gRPC.
    let mut client = OrchestratorServiceClient::connect(format!("http://{addr}"))
        .await
        .expect("failed to connect to orchestrator");

    // Drive the generated world end to end: register it, feed its inputs, and
    // step the engine until it reaches a terminal status.
    let (run_scope, hit_timeout) = execute_run(&mut client, &store, &mut world).await;

    // Now imagine the client runs the exact same thing again: the identical
    // blueprint (same schema, jobs, and inputs) under a brand-new run id.
    // Registration is a pure function of the workflow, so re-registering mints
    // the same ids — only the run id differs, giving the re-run its own stream.
    let mut rerun_world = world.rerun();
    let (rerun_scope, rerun_hit_timeout) = execute_run(&mut client, &store, &mut rerun_world).await;

    // Check invariants
    let initial_stream = Stream::new(store.clone(), run_scope);
    let initial_records = StreamReader::new(initial_stream).collect().await;
    if let Err(e) = initial_records {
        error!("Failed to collect initial records: {e}");
        return;
    }

    let rerun_stream = Stream::new(store.clone(), rerun_scope);
    let rerun_records = StreamReader::new(rerun_stream).collect().await;
    if let Err(e) = rerun_records {
        error!("Failed to collect rerun records: {e}");
        return;
    }

    let initial_records = initial_records.unwrap();
    let rerun_records = rerun_records.unwrap();

    let invariants: Vec<Box<dyn invariants::Invariant>> = vec![
        Box::new(invariants::CheckTerminalStatus::new(
            hit_timeout || rerun_hit_timeout,
        )),
        Box::new(invariants::CheckOrderedRunEvents::new(
            initial_records.clone(),
        )),
        Box::new(invariants::CheckOrderedRunEvents::new(
            rerun_records.clone(),
        )),
        Box::new(invariants::CheckIsReplayedEvents::new(
            rerun_records.clone(),
        )),
        Box::new(invariants::CheckReplayMatchesOriginal::new(
            initial_records.clone(),
            rerun_records.clone(),
        )),
    ];

    let runner = invariants::InvariantRunner::default();
    for invariant in &invariants {
        runner.run(invariant.as_ref());
    }
}

/// Register `world`, feed its inputs through the gRPC service, and drive a fresh
/// engine until it reaches a terminal status.
///
/// Returns the run scope the engine executed under and whether it gave up on the
/// [`IDLE_TIMEOUT`] instead of terminating cleanly.
async fn execute_run(
    client: &mut OrchestratorServiceClient<Channel>,
    store: &Store<MemoryStore>,
    world: &mut World,
) -> (RunScope, bool) {
    // Register the world as a workflow through the gRPC service.
    let registration_response = client
        .register_workflow(Request::new(world.register_request()))
        .await
        .expect("failed to register workflow")
        .into_inner();

    info!(
        "Registered world as workflow {} version {} ({} channels, {} jobs)",
        registration_response.workflow_id,
        registration_response.workflow_version_id,
        registration_response.channel_ids_by_name.len(),
        registration_response.job_ids_by_name.len()
    );

    // Fold the orchestrator-minted ids into the world before deriving anything
    // that depends on them.
    world.apply_registration(registration_response);

    // Generate input event(s) from the now-resolved ids and send them through
    // the gRPC service for ingestion.
    for request in world.input_event_requests() {
        client
            .handle_event(Request::new(request))
            .await
            .expect("failed to handle event");
        info!("Sent input event for ingestion");
    }

    // Step the engine until it's terminal, under the orchestrator-confirmed
    // run scope.
    let scope = world.run_scope();
    let mut engine = Engine::new(scope.clone(), store.clone())
        .await
        .expect("failed to create engine");

    let hit_timeout = drive_to_terminal(&mut engine).await;
    (scope, hit_timeout)
}

/// Step `engine` until it reports a terminal status, sleeping between idle polls.
///
/// Local jobs report back asynchronously (their scripts call the orchestrator
/// over gRPC), so the engine goes idle between making progress. We wait for
/// those response events instead of busy-spinning, and bail out if nothing
/// arrives for [`IDLE_TIMEOUT`] so a stuck run can't hang the harness forever.
///
/// Returns `true` if it bailed out on the idle timeout.
async fn drive_to_terminal<S: StorageProvider>(engine: &mut Engine<S>) -> bool {
    let mut last_progress = Instant::now();
    loop {
        match engine.step().await.expect("failed to step engine") {
            StepResult::Terminal(status) => {
                info!("Run reached terminal status: {status:?}");
                return false;
            }
            StepResult::Continue => {
                last_progress = Instant::now();
            }
            StepResult::Idle => {
                if last_progress.elapsed() >= IDLE_TIMEOUT {
                    warn!(
                        "Engine idle for {IDLE_TIMEOUT:?} without reaching a terminal status; stopping"
                    );
                    return true;
                }
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        }
    }
}
