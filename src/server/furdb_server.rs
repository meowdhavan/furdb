use std::net::{Ipv4Addr, SocketAddr};

use tokio::signal::unix::{signal, SignalKind};
use tonic::transport::server::TcpIncoming;
use tonic::transport::Server as TonicServer;

use crate::core::FurDB;
use crate::server::furdb_service::FurDbService;
use crate::server::proto::fur_db_server::FurDbServer;
use crate::server::server_config::ServerConfig;

use crate::error::ApplicationError;

#[derive(Clone)]
pub struct Server {
    server_config: ServerConfig,
    furdb: FurDB,
}

impl Server {
    pub fn new(server_config: ServerConfig, furdb: FurDB) -> Self {
        Self {
            server_config,
            furdb,
        }
    }

    pub async fn start(&self) -> Result<(), ApplicationError> {
        let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, self.server_config.port));
        // Binding here rather than letting `serve` do it lazily is what lets a
        // port clash surface as `ServerStart`. `TcpIncoming` leaves `nodelay`
        // unset though, so tonic's default has to be re-applied by hand —
        // without it Nagle pairs with delayed ACKs and every small unary call
        // pays tens of milliseconds.
        let incoming = TcpIncoming::bind(address)
            .map_err(|_| ApplicationError::ServerStart)?
            .with_nodelay(Some(true));

        let service = FurDbService::new(self.furdb.to_owned());

        log::info!("FurDB gRPC server listening on {address}");

        TonicServer::builder()
            .add_service(FurDbServer::new(service))
            .serve_with_incoming_shutdown(incoming, shutdown_signal())
            .await
            .map_err(|e| ApplicationError::Other(e.to_string()))?;

        Ok(())
    }
}

/// Resolves on `SIGINT` or `SIGTERM` so a request in flight finishes before the
/// process exits. A write killed midway would otherwise leave a table's `data`
/// and `sortfile` out of step, since deletes rewrite the whole file.
///
/// If a handler cannot be registered the server keeps serving rather than
/// shutting down — resolving on failure would make the process exit zero the
/// moment it started listening.
async fn shutdown_signal() {
    let handlers = signal(SignalKind::interrupt()).and_then(|interrupt| {
        let terminate = signal(SignalKind::terminate())?;
        Ok((interrupt, terminate))
    });

    let (mut interrupt, mut terminate) = match handlers {
        Ok(handlers) => handlers,
        Err(e) => {
            log::error!("Cannot listen for shutdown signals, they will be ignored: {e}");
            return std::future::pending().await;
        }
    };

    tokio::select! {
        _ = interrupt.recv() => log::info!("Received SIGINT, shutting down"),
        _ = terminate.recv() => log::info!("Received SIGTERM, shutting down"),
    }
}
