use std::{any::type_name, fmt, net::IpAddr, sync::Arc};

use tokio::signal;
use tracing::info;

use crate::wallet;

#[derive(Clone)]
pub struct GrpcController {
    pub list: Arc<dyn wallet::List>,
    pub balance: Arc<dyn wallet::Balance>,
    pub track: Arc<dyn wallet::Track>,
    pub untrack: Arc<dyn wallet::Untrack>,
}

impl fmt::Debug for GrpcController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(type_name::<Self>()).finish()
    }
}

#[derive(Debug, Clone)]
pub struct GrpcServer {
    controller: GrpcController,
    addr: Option<IpAddr>,
    port: Option<u16>,
}

impl GrpcServer {
    pub fn new(controller: GrpcController) -> Self {
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
