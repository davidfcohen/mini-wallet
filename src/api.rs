use std::{
    any::type_name,
    error, fmt,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use tokio::signal;
use tonic::transport::{Error as TransportError, Server};
use tonic_reflection::server::{Builder as ReflectionBuilder, Error as ReflectionError};
use tracing::info;

use crate::wallet;
use proto::FILE_DESCRIPTOR_SET;

#[derive(Debug)]
pub struct ApiError(Box<dyn error::Error + Send + Sync + 'static>);

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "couldn't start grpc server")
    }
}

impl error::Error for ApiError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}

impl From<TransportError> for ApiError {
    fn from(error: TransportError) -> Self {
        ApiError(error.into())
    }
}

impl From<ReflectionError> for ApiError {
    fn from(error: ReflectionError) -> Self {
        ApiError(error.into())
    }
}

#[derive(Debug)]
enum InnerError {
    Transport(TransportError),
    Reflection(ReflectionError),
}

#[derive(Clone)]
pub struct Controller {
    pub wallet_list: Arc<dyn wallet::List>,
    pub wallet_balance: Arc<dyn wallet::Balance>,
    pub wallet_track: Arc<dyn wallet::Track>,
    pub wallet_untrack: Arc<dyn wallet::Untrack>,
}

impl fmt::Debug for Controller {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(type_name::<Self>()).finish()
    }
}

#[derive(Debug, Clone)]
pub struct ApiServer {
    controller: Controller,
    addr: Option<IpAddr>,
    port: Option<u16>,
}

impl ApiServer {
    pub fn new(controller: Controller) -> Self {
        Self {
            controller,
            addr: None,
            port: None,
        }
    }

    pub fn with_addr(mut self, addr: IpAddr) -> Self {
        self.addr = Some(addr);
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub async fn run(self) -> Result<(), ApiError> {
        let addr = self.addr.unwrap_or_else(|| {
            info!("using default address");
            Ipv4Addr::new(0, 0, 0, 0).into()
        });

        let port = self.port.unwrap_or_else(|| {
            info!("using default port");
            50051
        });

        let reflection = ReflectionBuilder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()?;

        let socket = SocketAddr::new(addr, port);
        info!("started grpc server on {socket}");

        Server::builder()
            .add_service(reflection)
            .serve_with_shutdown(socket, capture_shutdown_signal())
            .await?;

        info!("exited with success");
        Ok(())
    }
}

async fn capture_shutdown_signal() {
    let interrupt = async {
        signal::ctrl_c()
            .await
            .expect("couldn't install SIGINT handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("couldn't install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = interrupt => {},
        _ = terminate => {},
    }

    info!("received shutdown signal")
}

mod proto {
    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("descriptor");
    tonic::include_proto!("wallet.v1");
}
