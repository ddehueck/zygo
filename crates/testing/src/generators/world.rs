//! The top of the generator hierarchy.
//!
//! Generation and *execution identity* are deliberately split into two types:
//!
//! - [`WorldBlueprint`] is the purely generated artifact: a self-contained,
//!   id-free description of a run (its [`workflow`](crate::generators::workflow)
//!   graph, the [`event`](crate::generators::event) inputs that drive it, the
//!   mode it runs under, and the one id the client legitimately owns — the run
//!   id). It is shaped like the requests the orchestrator accepts.
//!
//! - [`World`] wraps a blueprint and *accumulates* the real ids the orchestrator
//!   mints, as they come back in responses. Only the orchestrator can tell us
//!   the true channel/job/workflow ids, so [`World`] refuses to hand out a
//!   [`RunScope`] or input events until it has been told what registration
//!   produced.
//!
//! The intended drive loop:
//!
//! ```no_run
//! # use testing::generators::{world::{World, WorldGenerator}, GenerateExt};
//! let blueprint = WorldGenerator::default().generate_seeded(42);
//! let mut world = World::new(blueprint);
//! let _request = world.register_request();
//! // let response = client.register_workflow(_request).await?;
//! // world.apply_registration(response);
//! // for req in world.input_event_requests() { client.handle_event(req).await?; }
//! // let engine = Engine::new(world.run_scope(), store).await?;
//! ```

use std::collections::HashMap;

use rand_chacha::ChaCha8Rng;
use zygo_core::engine::RunScope;
use zygo_core::models::{
    ChannelId, ChannelName, DataReference, JobId, JobName, OrchestratorMode, RunId, WorkflowId,
    WorkflowVersionId,
};
use zygo_core::orchestrator_proto::{
    self, ChannelItemInsertedEvent, HandleEventRequest, JobRunEvent, RegisterWorkflowRequest,
    RegisterWorkflowResponse, RunId as ProtoRunId, job_run_event,
};
use uuid::Uuid;

use crate::generators::event::EventGenerator;
use crate::generators::workflow::{SOURCE_CHANNEL_NAME, WorkflowBlueprint, WorkflowGenerator};
use crate::generators::{Generate, choose, random_uuid};

/// A fully generated run, described entirely by intent: the mode it runs under,
/// the run id the client owns, its workflow graph, and the external inputs that
/// should be fed into it. Carries no orchestrator-minted ids.
#[derive(Debug, Clone)]
pub struct WorldBlueprint {
    /// The orchestrator mode this world is internally consistent with; the
    /// harness must run the orchestrator in this mode.
    pub mode: OrchestratorMode,
    /// The run's identity. The orchestrator does not mint run ids (a run is
    /// established implicitly by the first event), so this is the client's to
    /// choose and is kept seed-reproducible.
    pub run_id: RunId,
    /// The workflow graph to register.
    pub workflow: WorkflowBlueprint,
    /// External data references to insert into the source channel, one input
    /// event each.
    pub inputs: Vec<DataReference>,
}

impl WorldBlueprint {
    /// Clone this blueprint for a fresh run of the *identical* workflow graph
    /// and inputs, replacing only the client-owned run id.
    ///
    /// Everything the orchestrator turns into behaviour — the workflow schema,
    /// jobs, and input data references — is byte-for-byte unchanged; only the
    /// run identity differs. This models a client re-submitting the exact same
    /// work under a new run.
    pub fn with_run_id(&self, run_id: RunId) -> Self {
        Self {
            run_id,
            ..self.clone()
        }
    }
}

/// The orchestrator-confirmed identities for a registered workflow, folded in
/// from a [`RegisterWorkflowResponse`].
#[derive(Debug, Clone)]
struct Registration {
    workflow_id: WorkflowId,
    workflow_version_id: WorkflowVersionId,
    channel_ids_by_name: HashMap<ChannelName, ChannelId>,
    job_ids_by_name: HashMap<JobName, JobId>,
}

/// A blueprint plus the real ids accumulated from orchestrator responses.
///
/// Construct from a [`WorldBlueprint`], call [`World::register_request`] to get
/// the registration request, then [`World::apply_registration`] with the
/// response to unlock [`World::run_scope`] and [`World::input_event_requests`].
#[derive(Debug, Clone)]
pub struct World {
    blueprint: WorldBlueprint,
    registration: Option<Registration>,
}

impl World {
    /// Wrap a freshly generated blueprint. No ids are known yet.
    pub fn new(blueprint: WorldBlueprint) -> Self {
        Self {
            blueprint,
            registration: None,
        }
    }

    /// Begin a fresh, unregistered run of this world's *identical* blueprint
    /// (same schema, jobs, and inputs) under a new run id.
    ///
    /// Models a client running "the exact same thing" again. Because the
    /// orchestrator derives its ids purely from the workflow's names, registering
    /// the re-run mints the same workflow/channel/job ids — the run id is the
    /// only thing that differs, giving the re-run its own stream and run scope.
    ///
    /// The new run id is derived deterministically from the original, so the
    /// whole re-run scenario stays seed-reproducible while remaining distinct
    /// from the first run. The returned world is unregistered: call
    /// [`apply_registration`](Self::apply_registration) again before using its
    /// real ids.
    pub fn rerun(&self) -> Self {
        Self::new(self.blueprint.with_run_id(derive_rerun_run_id(&self.blueprint.run_id)))
    }

    /// The orchestrator mode this world must be run under.
    pub fn mode(&self) -> OrchestratorMode {
        self.blueprint.mode
    }

    /// Build the gRPC request to register this world's workflow.
    pub fn register_request(&self) -> RegisterWorkflowRequest {
        (&self.blueprint.workflow).into()
    }

    /// Fold the ids the orchestrator minted at registration into this world,
    /// unlocking [`run_scope`](Self::run_scope) and
    /// [`input_event_requests`](Self::input_event_requests).
    pub fn apply_registration(&mut self, response: RegisterWorkflowResponse) -> &mut Self {
        let channel_ids_by_name = response
            .channel_ids_by_name
            .into_iter()
            .map(|(name, id)| {
                let name = ChannelName::try_from(name).expect("valid channel name from server");
                let id = ChannelId::try_from(id).expect("valid channel id from server");
                (name, id)
            })
            .collect();

        let job_ids_by_name = response
            .job_ids_by_name
            .into_iter()
            .map(|(name, id)| {
                let name = JobName::try_from(name).expect("valid job name from server");
                let id = JobId::try_from(id).expect("valid job id from server");
                (name, id)
            })
            .collect();

        self.registration = Some(Registration {
            workflow_id: WorkflowId::try_from(response.workflow_id)
                .expect("valid workflow id from server"),
            workflow_version_id: WorkflowVersionId::try_from(response.workflow_version_id)
                .expect("valid workflow version id from server"),
            channel_ids_by_name,
            job_ids_by_name,
        });

        self
    }

    /// The identity triple this run executes under, built from the orchestrator's
    /// confirmed ids plus the client-owned run id.
    ///
    /// Panics if [`apply_registration`](Self::apply_registration) has not been
    /// called.
    pub fn run_scope(&self) -> RunScope {
        let registration = self.registration();
        RunScope::new(
            registration.workflow_id.clone(),
            registration.workflow_version_id.clone(),
            self.blueprint.run_id.clone(),
        )
    }

    /// Build the external input events that insert this world's generated data
    /// references into the registered source channel, kicking off the run.
    ///
    /// Panics if [`apply_registration`](Self::apply_registration) has not been
    /// called.
    pub fn input_event_requests(&self) -> Vec<HandleEventRequest> {
        let registration = self.registration();

        let source_channel_name =
            ChannelName::try_from(SOURCE_CHANNEL_NAME.to_owned()).expect("valid channel name");
        let source_channel_id = registration
            .channel_ids_by_name
            .get(&source_channel_name)
            .expect("source channel registered");

        let run_id = Some(ProtoRunId {
            workflow_id: registration.workflow_id.as_ref().to_string(),
            workflow_version_id: registration.workflow_version_id.as_ref().to_string(),
            workflow_run_id: self.blueprint.run_id.as_ref().to_string(),
        });

        self.blueprint
            .inputs
            .iter()
            // For now, only generate a single input event per world.
            .take(1)
            .map(|input| HandleEventRequest {
                event: Some(JobRunEvent {
                    id: Uuid::now_v7().to_string(),
                    run_id: run_id.clone(),
                    source: Some(job_run_event::Source::InputSource(
                        orchestrator_proto::InputSource {},
                    )),
                    event: Some(job_run_event::Event::ChannelItemInserted(
                        ChannelItemInsertedEvent {
                            channel_id: source_channel_id.as_ref().to_string(),
                            data_reference: Some(orchestrator_proto::DataReference {
                                uri: input.uri.clone(),
                                etag: input.etag.clone(),
                                content_type: input.content_type.clone(),
                                size_bytes: input.size_bytes,
                            }),
                        },
                    )),
                }),
            })
            .collect()
    }

    /// The orchestrator-minted id for a channel, by name. `None` until
    /// registration, or if no such channel was registered.
    pub fn channel_id(&self, name: &ChannelName) -> Option<&ChannelId> {
        self.registration
            .as_ref()
            .and_then(|registration| registration.channel_ids_by_name.get(name))
    }

    /// The orchestrator-minted id for a job, by name. `None` until registration,
    /// or if no such job was registered.
    pub fn job_id(&self, name: &JobName) -> Option<&JobId> {
        self.registration
            .as_ref()
            .and_then(|registration| registration.job_ids_by_name.get(name))
    }

    fn registration(&self) -> &Registration {
        self.registration
            .as_ref()
            .expect("world must be registered before its real ids can be used")
    }
}

/// Derive a distinct but deterministic run id from an existing one.
///
/// A namespaced (v5) UUID is a pure function of its input, so a re-run of the
/// same blueprint stays seed-reproducible, yet it is guaranteed to differ from
/// the run id it was derived from.
fn derive_rerun_run_id(run_id: &RunId) -> RunId {
    let derived = Uuid::new_v5(&Uuid::NAMESPACE_OID, run_id.as_ref().as_bytes());
    RunId::try_from(derived.to_string()).expect("derived run id is non-empty")
}

/// Describes how a whole [`WorldBlueprint`] is generated by composing the
/// sub-generators.
#[derive(Debug, Clone)]
pub struct WorldGenerator {
    /// The orchestrator modes a world may be generated for. One is drawn per
    /// world so the entire graph stays internally consistent.
    ///
    /// Defaults to [`OrchestratorMode::Local`] only: remote entrypoints make the
    /// engine perform real HTTP calls, which breaks the determinism DST relies
    /// on. Add [`OrchestratorMode::Remote`] here once the remote transport can
    /// be simulated.
    pub modes: Vec<OrchestratorMode>,
    /// How the workflow graph is generated.
    pub workflow: WorkflowGenerator,
    /// How the external inputs are generated.
    pub event: EventGenerator,
}

impl Default for WorldGenerator {
    fn default() -> Self {
        Self {
            modes: vec![OrchestratorMode::Local], // Only care about local for now.
            workflow: WorkflowGenerator::default(),
            event: EventGenerator::default(),
        }
    }
}

impl Generate for WorldGenerator {
    type Output = WorldBlueprint;
    type Context = ();

    fn generate(&self, rng: &mut ChaCha8Rng, _context: ()) -> WorldBlueprint {
        let mode = *choose(rng, &self.modes);
        let run_id = RunId::try_from(random_uuid(rng).to_string()).expect("valid run id");
        let workflow = self.workflow.generate(rng, mode);
        let inputs = self.event.generate(rng, ());
        WorldBlueprint {
            mode,
            run_id,
            workflow,
            inputs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::GenerateExt;
    use crate::generators::workflow::WORKFLOW_NAME;
    use std::collections::HashSet;

    #[test]
    fn generation_is_deterministic_for_a_seed() {
        let generator = WorldGenerator::default();
        let first = generator.generate_seeded(123);
        let second = generator.generate_seeded(123);
        assert_eq!(format!("{first:?}"), format!("{second:?}"));
    }

    #[test]
    fn register_request_declares_a_source_channel_and_inputs() {
        let blueprint = WorldGenerator::default().generate_seeded(99);
        let request = RegisterWorkflowRequest::from(&blueprint.workflow);

        assert_eq!(request.name, WORKFLOW_NAME);
        assert!(!request.jobs.is_empty());
        assert!(
            request
                .channels
                .iter()
                .any(|channel| channel.name == SOURCE_CHANNEL_NAME)
        );

        // Every job resolves to a declared input channel.
        let declared: HashSet<&str> = request
            .channels
            .iter()
            .map(|channel| channel.name.as_str())
            .collect();
        for job in &request.jobs {
            assert!(declared.contains(job.input_channel_name.as_str()));
        }

        assert!(!blueprint.inputs.is_empty());
        assert!(blueprint.inputs.iter().all(|input| !input.uri.trim().is_empty()));
    }

    #[test]
    fn rerun_keeps_the_blueprint_but_changes_the_run_id() {
        let blueprint = WorldGenerator::default().generate_seeded(7);
        let world = World::new(blueprint.clone());
        let rerun = world.rerun();

        // The re-run shares the exact same workflow graph and inputs...
        assert_eq!(
            format!("{:?}", rerun.blueprint.workflow),
            format!("{:?}", blueprint.workflow)
        );
        assert_eq!(
            format!("{:?}", rerun.blueprint.inputs),
            format!("{:?}", blueprint.inputs)
        );
        assert_eq!(rerun.blueprint.mode, blueprint.mode);

        // ...but runs under a brand-new, distinct run id.
        assert_ne!(
            rerun.blueprint.run_id.as_ref(),
            blueprint.run_id.as_ref(),
            "a re-run must not reuse the original run id"
        );

        // Deriving the run id is deterministic, keeping the scenario reproducible.
        let rerun_again = World::new(blueprint).rerun();
        assert_eq!(
            rerun.blueprint.run_id.as_ref(),
            rerun_again.blueprint.run_id.as_ref()
        );
    }

    #[test]
    fn world_uses_orchestrator_ids_after_registration() {
        let blueprint = WorldGenerator::default().generate_seeded(7);
        let mut world = World::new(blueprint);

        // A registration response whose ids deliberately differ from the names,
        // so we can prove the world adopts the server's ids rather than any it
        // might have fabricated.
        let request = world.register_request();
        let channel_ids_by_name = request
            .channels
            .iter()
            .map(|channel| (channel.name.clone(), format!("cid-{}", channel.name)))
            .collect();
        let job_ids_by_name = request
            .jobs
            .iter()
            .map(|job| (job.name.clone(), format!("jid-{}", job.name)))
            .collect();

        world.apply_registration(RegisterWorkflowResponse {
            workflow_id: "wf-real".to_string(),
            workflow_version_id: "ver-real".to_string(),
            channel_ids_by_name,
            job_ids_by_name,
        });

        let scope = world.run_scope();
        assert_eq!(scope.workflow_id.as_ref(), "wf-real");
        assert_eq!(scope.workflow_version_id.as_ref(), "ver-real");

        let requests = world.input_event_requests();
        assert!(!requests.is_empty());
        for request in &requests {
            let event = request.event.as_ref().expect("event present");
            let run_id = event.run_id.as_ref().expect("run id present");
            assert_eq!(run_id.workflow_id, "wf-real");
            assert_eq!(run_id.workflow_version_id, "ver-real");
            assert_eq!(run_id.workflow_run_id, scope.run_id.as_ref());

            match event.event.as_ref().expect("event kind present") {
                job_run_event::Event::ChannelItemInserted(inserted) => {
                    assert_eq!(inserted.channel_id, "cid-source");
                }
                _ => panic!("expected a channel item inserted event"),
            }
        }
    }
}
