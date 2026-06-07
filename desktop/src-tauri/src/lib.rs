mod grpc_client;
mod models;

use grpc_client::Client;
use models::{
    Channel, Edge, Event, GetWorkflowVersionSchemaParams, Job, ListRunEventsParams, ListRunsParams,
    ListWorkflowsParams, LivestreamParams, LivestreamResponse, Run, Workflow, WorkflowVersionSchema,
};
use specta_typescript::{BigIntExportBehavior, Typescript};
use std::sync::Arc;
use tauri_specta::{collect_commands, Builder};
use tokio::sync::Mutex;

const GRPC_SERVER_ADDR: &str = "http://localhost:50051";

struct AppState {
    client: Arc<Mutex<Option<Client>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
        }
    }
}

#[tauri::command]
#[specta::specta]
async fn list_workflows(
    state: tauri::State<'_, AppState>,
    params: ListWorkflowsParams,
) -> Result<Vec<Workflow>, String> {
    let mut guard = state.client.lock().await;
    if guard.is_none() {
        let client = Client::connect(GRPC_SERVER_ADDR)
            .await
            .map_err(|e| format!("Failed to connect to gRPC server: {}", e))?;
        *guard = Some(client);
    }
    let client = guard.as_mut().unwrap();

    let response = client
        .list_workflows(
            params.workflow_id,
            params.sort.unwrap_or_default().into(),
            params.limit.unwrap_or(50),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(response.workflows.into_iter().map(Workflow::from).collect())
}

#[tauri::command]
#[specta::specta]
async fn list_runs(
    state: tauri::State<'_, AppState>,
    params: ListRunsParams,
) -> Result<Vec<Run>, String> {
    let mut guard = state.client.lock().await;
    if guard.is_none() {
        let client = Client::connect(GRPC_SERVER_ADDR)
            .await
            .map_err(|e| format!("Failed to connect to gRPC server: {}", e))?;
        *guard = Some(client);
    }
    let client = guard.as_mut().unwrap();

    let response = client
        .list_runs(
            params.workflow_id,
            params.run_id,
            params.sort.unwrap_or_default().into(),
            params.limit.unwrap_or(50),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(response.runs.into_iter().map(Run::from).collect())
}

#[tauri::command]
#[specta::specta]
async fn list_run_events(
    state: tauri::State<'_, AppState>,
    params: ListRunEventsParams,
) -> Result<Vec<Event>, String> {
    let mut guard = state.client.lock().await;
    if guard.is_none() {
        let client = Client::connect(GRPC_SERVER_ADDR)
            .await
            .map_err(|e| format!("Failed to connect to gRPC server: {}", e))?;
        *guard = Some(client);
    }
    let client = guard.as_mut().unwrap();

    let response = client
        .list_run_events(
            params.run_id,
            params.sequence_number,
            params.sort.unwrap_or_default().into(),
            params.limit.unwrap_or(100),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(response.events.into_iter().map(Event::from).collect())
}

#[tauri::command]
#[specta::specta]
async fn get_workflow_version_schema(
    state: tauri::State<'_, AppState>,
    params: GetWorkflowVersionSchemaParams,
) -> Result<WorkflowVersionSchema, String> {
    let mut guard = state.client.lock().await;
    if guard.is_none() {
        let client = Client::connect(GRPC_SERVER_ADDR)
            .await
            .map_err(|e| format!("Failed to connect to gRPC server: {}", e))?;
        *guard = Some(client);
    }
    let client = guard.as_mut().unwrap();

    let response = client
        .get_workflow_version_schema(params.workflow_version_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(WorkflowVersionSchema {
        jobs: response.jobs.into_iter().map(Job::from).collect(),
        channels: response.channels.into_iter().map(Channel::from).collect(),
        edges: response.edges.into_iter().map(Edge::from).collect(),
    })
}

#[tauri::command]
#[specta::specta]
async fn livestream(
    state: tauri::State<'_, AppState>,
    params: LivestreamParams,
) -> Result<LivestreamResponse, String> {
    let mut guard = state.client.lock().await;
    if guard.is_none() {
        let client = Client::connect(GRPC_SERVER_ADDR)
            .await
            .map_err(|e| format!("Failed to connect to gRPC server: {}", e))?;
        *guard = Some(client);
    }
    let client = guard.as_mut().unwrap();

    let response = client
        .livestream(params.cursor.map(|c| c.into()))
        .await
        .map_err(|e| e.to_string())?;

    Ok(LivestreamResponse {
        workflows: response.workflows.into_iter().map(|w| w.into()).collect(),
        runs: response.runs.into_iter().map(|r| r.into()).collect(),
        events: response.events.into_iter().map(|e| e.into()).collect(),
        next_cursor: response.next_cursor.map(|c| c.into()),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        list_workflows,
        list_runs,
        list_run_events,
        get_workflow_version_schema,
        livestream,
    ]);

    #[cfg(debug_assertions)]
    builder
        .export(
            Typescript::default().bigint(BigIntExportBehavior::BigInt),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
