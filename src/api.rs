use std::{
    any::type_name,
    error, fmt,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use async_trait::async_trait;
use tokio::signal;
use tonic::{
    Request, Response, Result, Status,
    transport::{Error as TransportError, Server as InnerServer},
};
use tonic_reflection::server::{Builder as ReflectionBuilder, Error as ReflectionError};
use tracing::info;

use crate::wallet::{self, WalletError, WalletErrorKind};
use proto::{
    BalanceRequest, BalanceResponse, FILE_DESCRIPTOR_SET, ListResponse, TrackRequest,
    UntrackRequest, Wallet,
    wallet_service_server::{WalletService, WalletServiceServer},
};

mod proto {
    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("descriptor");
    tonic::include_proto!("wallet.v1");
}

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
pub struct Server {
    controller: Controller,
    addr: Option<IpAddr>,
    port: Option<u16>,
}

impl Server {
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

        let server_reflection = ReflectionBuilder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()?;

        let server_wallet = WalletServiceServer::new(WalletServer {
            controller: self.controller,
        });

        let socket = SocketAddr::new(addr, port);
        info!("started grpc server on {socket}");

        InnerServer::builder()
            .add_service(server_reflection)
            .add_service(server_wallet)
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

#[derive(Debug, Clone)]
pub struct WalletServer {
    pub controller: Controller,
}

#[async_trait]
impl WalletService for WalletServer {
    async fn balance(&self, request: Request<BalanceRequest>) -> Result<Response<BalanceResponse>> {
        let name = request
            .into_inner()
            .name
            .ok_or(Status::invalid_argument("missing required name"))?;

        let balance = self
            .controller
            .wallet_balance
            .execute(&name)
            .await
            .map_err(|e| error_to_status(&e))?;

        Ok(Response::new(BalanceResponse {
            balance: Some(balance),
        }))
    }

    async fn list(&self, _request: Request<()>) -> Result<Response<ListResponse>> {
        let wallets = self
            .controller
            .wallet_list
            .execute()
            .await
            .map_err(|e| error_to_status(&e))?;

        let wallets = wallets
            .into_iter()
            .map(|w| Wallet {
                name: Some(w.name),
                address: Some(w.address),
            })
            .collect();

        Ok(Response::new(ListResponse { wallet: wallets }))
    }

    async fn track(&self, request: Request<TrackRequest>) -> Result<Response<()>> {
        let name = request
            .into_inner()
            .name
            .ok_or(Status::invalid_argument("missing required name"))?;

        let address = request
            .into_inner()
            .address
            .ok_or(Status::invalid_argument("missing required address"))?;

        self.controller
            .wallet_track
            .execute(&name, &address)
            .await
            .map_err(|e| error_to_status(&e))?;

        Ok(Response::new(()))
    }

    async fn untrack(&self, request: Request<UntrackRequest>) -> Result<Response<()>> {
        let name = request
            .into_inner()
            .name
            .ok_or(Status::invalid_argument("missing required name"))?;

        self.controller
            .wallet_untrack
            .execute(&name)
            .await
            .map_err(|e| error_to_status(&e))?;

        Ok(Response::new(()))
    }
}

fn error_to_status(error: &WalletError) -> Status {
    let message = compose_error(error);

    match error.kind() {
        WalletErrorKind::NotFound => Status::not_found(message),
        WalletErrorKind::NameConflict => Status::already_exists(message),
        WalletErrorKind::NameEmpty => Status::invalid_argument(message),
        WalletErrorKind::NameTooLong => Status::invalid_argument(message),
        WalletErrorKind::WalletStore => Status::internal(message),
        WalletErrorKind::WalletChain => Status::internal(message),
        WalletErrorKind::WalletAddrParse => Status::invalid_argument(message),
    }
}

fn compose_error(error: &dyn std::error::Error) -> String {
    let mut composed = error.to_string();

    let mut next: &dyn std::error::Error = &error;
    while let Some(source) = next.source() {
        composed.push_str(": ");
        composed.push_str(&source.to_string());
        next = source;
    }

    composed
}

fn missing_required(arg: &str) -> Status {
    Status::invalid_argument(format!("missing required {arg}"))
}
