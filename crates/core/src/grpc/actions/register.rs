use crate::{
    models::{
        Channel, ChannelId, ChannelName, ContentHash, Edge, EdgeKind, Job, JobId, JobName,
        Workflow, WorkflowVersion, WorkflowVersionSchema,
    },
    orchestrator_proto::{RegisterWorkflowRequest, RegisterWorkflowResponse},
    store::{StorageProvider, Store},
};

pub async fn register_workflow<S: StorageProvider>(
    store: &Store<S>,
    input: RegisterWorkflowRequest,
) -> anyhow::Result<RegisterWorkflowResponse> {
    let workflow = Workflow {
        id: input.name.clone().try_into()?,
        name: input.name.try_into()?,
    };

    let channels = input
        .channels
        .into_iter()
        .map(|channel| {
            Ok(Channel {
                id: ChannelId::try_from(channel.name.clone())?,
                name: ChannelName::try_from(channel.name.clone())?,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let mut edges = Vec::new();
    let mut jobs = Vec::new();

    for job_schema in &input.jobs {
        let job = Job {
            id: JobId::try_from(job_schema.name.clone())?,
            name: JobName::try_from(job_schema.name.clone())?,
            content_hash: ContentHash::try_from(job_schema.content_hash.clone())?,
            entrypoint: job_schema
                .entrypoint
                .clone()
                .ok_or_else(|| anyhow::anyhow!("job entrypoint is required"))?
                .try_into()?,
        };

        let input_channel = channels
            .iter()
            .find(|c| c.name.as_ref() == job_schema.input_channel_name);

        if let Some(input_channel) = input_channel {
            edges.push(Edge {
                job_id: job.id.clone(),
                channel_id: input_channel.id.clone(),
                kind: EdgeKind::Input,
            });
        } else {
            return Err(anyhow::anyhow!(
                "Input channel not found for job: {}",
                job_schema.input_channel_name
            ));
        }

        for output_channel_name in &job_schema.output_channel_names {
            let output_channel = channels
                .iter()
                .find(|c| c.name.as_ref() == output_channel_name);
            if let Some(output_channel) = output_channel {
                edges.push(Edge {
                    job_id: job.id.clone(),
                    channel_id: output_channel.id.clone(),
                    kind: EdgeKind::Output,
                });
            } else {
                return Err(anyhow::anyhow!(
                    "Output channel not found for job: {}",
                    output_channel_name
                ));
            }
        }

        jobs.push(job);
    }

    let channel_ids_by_name = channels
        .iter()
        .map(|c| (c.name.clone().to_string(), c.id.clone().to_string()))
        .collect();

    let job_ids_by_name = jobs
        .iter()
        .map(|j| (j.name.clone().to_string(), j.id.clone().to_string()))
        .collect();

    let content_hash = ContentHash::try_from(input.content_hash.clone())?;
    let schema = WorkflowVersionSchema {
        jobs,
        channels,
        edges,
    };

    let workflow_version = WorkflowVersion {
        id: input.content_hash.try_into()?,
        workflow_id: workflow.id.clone(),
        content_hash,
        schema,
    };

    store.workflows().put(&workflow).await?;
    store.versions().put(&workflow_version).await?;

    Ok(RegisterWorkflowResponse {
        workflow_id: workflow.id.clone().to_string(),
        workflow_version_id: workflow_version.id.clone().to_string(),
        channel_ids_by_name,
        job_ids_by_name,
    })
}
