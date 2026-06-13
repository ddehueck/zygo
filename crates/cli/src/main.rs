use std::ffi::OsString;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Context;
use clap::{Parser, Subcommand};
use service_manager::{
    RestartPolicy, ServiceInstallCtx, ServiceLabel, ServiceLevel, ServiceManager, ServiceStartCtx,
    ServiceStatus, ServiceStatusCtx, ServiceStopCtx, ServiceUninstallCtx,
};
use tracing::info;
use zygo_core::grpc::OrchestratorService;
use zygo_core::models::OrchestratorMode;
use zygo_core::store::{MemoryStore, Store};

const SERVICE_LABEL: &str = "com.zygo.background";
const DEFAULT_ORCHESTRATOR_ADDR: &str = "127.0.0.1:50051";

#[derive(Parser)]
#[command(name = "zygo", about = "Zygo CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Install the zygo background service
    Install,
    /// Uninstall the zygo background service
    Uninstall,
    /// Start the installed background service and wait for the orchestrator gRPC server
    Start,
    /// Stop the installed background service and its orchestrator gRPC server
    Stop,
    /// Get the status of the installed background service
    Status,
    /// Run the orchestrator gRPC server (invoked by the OS background service)
    #[command(hide = true)]
    Run,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Install => {
            let label = service_label()?;
            let manager = service_manager()?;
            install(&*manager, label)
        }
        Command::Uninstall => {
            let label = service_label()?;
            let manager = service_manager()?;
            uninstall(&*manager, label)
        }
        Command::Start => {
            let label = service_label()?;
            let manager = service_manager()?;
            start_service(&*manager, label)
        }
        Command::Stop => {
            let label = service_label()?;
            let manager = service_manager()?;
            stop_service(&*manager, label)
        }
        Command::Status => {
            let label = service_label()?;
            let manager = service_manager()?;
            status(&*manager, label)
        }
        Command::Run => run_orchestrator(),
    }
}

fn service_label() -> anyhow::Result<ServiceLabel> {
    SERVICE_LABEL
        .parse()
        .context("failed to parse service label")
}

fn service_manager() -> anyhow::Result<Box<dyn ServiceManager>> {
    let mut manager = <dyn ServiceManager>::native().context("no native service manager")?;
    manager
        .set_level(ServiceLevel::User)
        .context("failed to set user service level")?;
    Ok(manager)
}

fn install(manager: &dyn ServiceManager, label: ServiceLabel) -> anyhow::Result<()> {
    let program = std::env::current_exe().context("failed to resolve current executable")?;

    manager
        .install(ServiceInstallCtx {
            label,
            program,
            args: vec![OsString::from("run")],
            contents: None,
            username: None,
            working_directory: None,
            environment: None,
            autostart: true,
            // Manual start/stop via the CLI; KeepAlive would cause launchd to respawn
            // the process after `launchctl stop`.
            restart_policy: RestartPolicy::Never,
        })
        .context("failed to install service")?;

    println!("Installed {SERVICE_LABEL}");
    Ok(())
}

fn uninstall(manager: &dyn ServiceManager, label: ServiceLabel) -> anyhow::Result<()> {
    manager
        .uninstall(ServiceUninstallCtx { label })
        .context("failed to uninstall service")?;

    println!("Uninstalled {SERVICE_LABEL}");
    Ok(())
}

fn start_service(manager: &dyn ServiceManager, label: ServiceLabel) -> anyhow::Result<()> {
    manager
        .start(ServiceStartCtx { label })
        .context("failed to start service")?;

    let addr = orchestrator_addr()?;
    wait_for_orchestrator(addr).context("orchestrator gRPC server did not become ready")?;

    println!("Started {SERVICE_LABEL} — orchestrator listening on http://{addr}");
    Ok(())
}

fn stop_service(manager: &dyn ServiceManager, label: ServiceLabel) -> anyhow::Result<()> {
    manager
        .stop(ServiceStopCtx { label })
        .context("failed to stop service")?;

    let addr = orchestrator_addr()?;
    wait_for_orchestrator_shutdown(addr)
        .context("orchestrator gRPC server did not shut down")?;

    println!("Stopped {SERVICE_LABEL} — orchestrator gRPC server is no longer running");
    Ok(())
}

fn status(manager: &dyn ServiceManager, label: ServiceLabel) -> anyhow::Result<()> {
    let status = manager.status(ServiceStatusCtx {
        label: label.clone(),
    })?;

    println!("Status: {status:?}");
    Ok(())
}

fn run_orchestrator() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to create tokio runtime")?
        .block_on(serve_orchestrator())
}

async fn serve_orchestrator() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let addr = orchestrator_addr()?;
    let mode = OrchestratorMode::from_env();
    let store = Store::new(MemoryStore::new());
    let service = OrchestratorService::new(store, mode);
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    info!("starting orchestrator on {addr} (mode: {mode})");

    tokio::spawn(async move {
        wait_for_shutdown_signal().await;
        shutdown_tx.send(()).ok();
    });

    service
        .serve_with_shutdown(addr, shutdown_rx)
        .await
        .map_err(|err| anyhow::anyhow!("orchestrator server failed: {err}"))?;

    Ok(())
}

async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = sigterm.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await.ok();
    }
}

fn orchestrator_addr() -> anyhow::Result<SocketAddr> {
    std::env::var("ORCHESTRATOR_ADDR")
        .unwrap_or_else(|_| DEFAULT_ORCHESTRATOR_ADDR.to_string())
        .parse()
        .context("invalid ORCHESTRATOR_ADDR")
}

fn wait_for_orchestrator(addr: SocketAddr) -> anyhow::Result<()> {
    const MAX_ATTEMPTS: u32 = 100;
    const RETRY_DELAY: Duration = Duration::from_millis(100);

    for _ in 0..MAX_ATTEMPTS {
        if std::net::TcpStream::connect(addr).is_ok() {
            return Ok(());
        }
        std::thread::sleep(RETRY_DELAY);
    }

    anyhow::bail!("timed out waiting for orchestrator at {addr}");
}

fn wait_for_orchestrator_shutdown(addr: SocketAddr) -> anyhow::Result<()> {
    const MAX_ATTEMPTS: u32 = 100;
    const RETRY_DELAY: Duration = Duration::from_millis(100);

    for _ in 0..MAX_ATTEMPTS {
        if std::net::TcpStream::connect(addr).is_err() {
            return Ok(());
        }
        std::thread::sleep(RETRY_DELAY);
    }

    anyhow::bail!("timed out waiting for orchestrator to shut down at {addr}");
}
