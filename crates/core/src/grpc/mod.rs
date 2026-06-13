//! gRPC transport for the orchestrator.
//!
//! This module hosts the [`OrchestratorService`] implementation of the
//! generated `OrchestratorService` gRPC trait, along with helpers to run a
//! Tonic server. The service is constructed from a caller-provided [`Store`],
//! so embedders are responsible for wiring up storage (and an
//! [`OrchestratorMode`]) and then calling [`OrchestratorService::serve`].
//!
//! [`Store`]: crate::stores::Store
//! [`OrchestratorMode`]: crate::models::OrchestratorMode

mod actions;
mod convert;
mod parse;
mod service;
mod validate;

pub use parse::{ChannelSchemaInput, JobSchemaInput, RegisterWorkflowInput};
pub use service::OrchestratorService;

use std::net::SocketAddr;

use tokio::sync::oneshot;
use tonic::transport::{Server, server::Router};

use crate::store::StorageProvider;

use crate::orchestrator_proto::orchestrator_service_server::OrchestratorServiceServer;

/// Encoded proto file descriptor set, emitted by `build.rs`. Used to enable
/// gRPC reflection in debug builds so tooling like `grpcurl` can discover
/// services and methods automatically.
#[cfg(debug_assertions)]
const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("orchestrator_descriptor");

impl<S: StorageProvider + 'static> OrchestratorService<S> {
    /// Build a Tonic [`Router`] hosting this service.
    ///
    /// In debug builds, gRPC reflection is also registered. Use this when you
    /// need to compose additional services onto the same server; otherwise
    /// prefer [`OrchestratorService::serve`].
    pub fn into_router(self) -> Result<Router, Box<dyn std::error::Error>> {
        #[cfg(debug_assertions)]
        let builder = {
            let reflection_service = tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                .build_v1()?;

            Server::builder().add_service(reflection_service)
        };

        #[cfg(not(debug_assertions))]
        let builder = Server::builder();

        Ok(builder.add_service(OrchestratorServiceServer::new(self)))
    }

    /// Run the gRPC server on `addr` until `shutdown_rx` receives a signal.
    pub async fn serve_with_shutdown(
        self,
        addr: SocketAddr,
        shutdown_rx: oneshot::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.into_router()?
            .serve_with_shutdown(addr, async move {
                shutdown_rx.await.ok();
            })
            .await?;
        Ok(())
    }
}
