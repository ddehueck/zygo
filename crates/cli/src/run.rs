use std::process::Command;

use zygo_core::{
    MemoryStore, Zygo, ZygoConfig,
    models::{DataReference, LocalEntrypoint},
    store::Store,
};

use crate::python::{WorkflowMetadata, workflow_schema_from_metadata};

const ZYGO_PKG_INTERNAL_CLI_MODULE: &str = "zygo._internal.ipc.v0";

pub async fn run_workflow(
    target: &str,
    fsspec_uri: &str,
    workers: Option<usize>,
) -> anyhow::Result<()> {
    // 1. Find the current python executable in the current working directory
    // Start with `uv python` for now
    let python = Command::new("uv").args(["python", "find"]).output()?;
    anyhow::ensure!(
        python.status.success(),
        "Could not find a python executable"
    );
    let python = String::from_utf8_lossy(&python.stdout).trim().to_owned();
    println!("{python}");

    // 2. Ensure that the zygo package is in the executable's environment
    let package = Command::new(&python).args(["-c", "import zygo"]).status()?;
    anyhow::ensure!(package.success(), "zygo is not available in {python}");
    println!("zygo is available in {python}");

    // 3. Use the zygo package to inspect the workflow to build the schema
    let metadata = Command::new(&python)
        .args(["-m", ZYGO_PKG_INTERNAL_CLI_MODULE, "metadata", target])
        .output()?;
    anyhow::ensure!(
        metadata.status.success(),
        "Failed to get metadata for {target}: {}",
        String::from_utf8_lossy(&metadata.stderr).trim()
    );
    let metadata = String::from_utf8_lossy(&metadata.stdout).trim().to_owned();
    println!("metadata: {metadata}");

    // 3.5 Parse the metadata into a structured format
    let metadata: WorkflowMetadata = serde_json::from_str(&metadata)?;
    println!("parsed metadata: {metadata:?}");

    let base_entrypoint = LocalEntrypoint {
        cwd: std::env::current_dir()?.to_string_lossy().into_owned(),
        exec: python,
        args: vec!["-m".into(), ZYGO_PKG_INTERNAL_CLI_MODULE.into()],
    };
    let schema = workflow_schema_from_metadata(metadata, base_entrypoint)?;
    println!("built workflow schema: {schema:?}");

    // 4. Create a zygo service and start the workflow
    let num_workers = workers.unwrap_or(1); // TODO: Use CPU core count
    let config = ZygoConfig::new(num_workers);
    let store = Store::new(MemoryStore::new()); // TODO: This is a lil funky wording

    let service = Zygo::new(store, config);

    let data_ref = DataReference {
        uri: fsspec_uri.to_string(),
        etag: "42".into(),
        content_type: None,
        size_bytes: None,
    };

    let run_id = service.run(data_ref, schema).await?;
    println!("run_id: {run_id:?}");

    // 5. Watch the engine state and log it to stdout
    let mut rx = service.subscribe(&run_id).await?;
    loop {
        let snapshot = rx.borrow_and_update().clone();
        println!("engine snapshot: {}", serde_json::to_string(&snapshot)?);

        if snapshot.state.status.is_terminal() {
            break;
        }

        rx.changed().await.map_err(|_| {
            anyhow::anyhow!("workflow actor stopped before reaching a terminal state")
        })?;
    }

    Ok(())
}
