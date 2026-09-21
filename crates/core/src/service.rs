use crate::actor::{ActorHandle, ActorStateRx};
use crate::context::RunContext;
use crate::dependencies::AppDeps;
use crate::models::{
    ChannelItemInsertedData, DataReferenceUri, Event, EventId, EventKind, Source, WorkflowRunId,
    WorkflowSchema,
};
use std::time::SystemTime;

pub struct ZygoRun {
    pub id: WorkflowRunId,
    actor: ActorHandle,
}

impl ZygoRun {
    pub async fn start<D: AppDeps>(
        id: &WorkflowRunId,
        inputs: Vec<DataReferenceUri>,
        schema: WorkflowSchema,
        deps: D,
    ) -> Result<Self, anyhow::Error> {
        anyhow::ensure!(
            !inputs.is_empty(),
            "a workflow run requires at least one input"
        );

        let input_events = inputs
            .into_iter()
            .map(|input| Event {
                id: EventId::new(),
                is_replay: false,
                timestamp: SystemTime::now(),
                kind: EventKind::ChannelItemInserted(ChannelItemInsertedData {
                    channel_id: schema.input_channel_id.clone(),
                    item: input,
                }),
                source: Source::Input,
                run_id: id.clone(),
            })
            .collect();

        let run_context = RunContext {
            deps,
            run_id: id.clone(),
            schema,
        };

        let actor = ActorHandle::spawn(&run_context, input_events).await?;

        Ok(Self {
            id: id.clone(),
            actor,
        })
    }

    pub async fn cancel(&self) -> Result<(), anyhow::Error> {
        self.actor.cancel().await;
        Ok(())
    }

    pub fn subscribe(&self) -> Result<ActorStateRx, anyhow::Error> {
        Ok(self.actor.state_rx.clone())
    }
}
