#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

use std::{error::Error, process, sync::Arc};

use mini_wallet::{
    fs::FsWalletStore,
    rpc::RpcWalletClient,
    server::{Controller, Server},
    wallet,
};

use tracing::error;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone)]
struct Dependencies {
    wallet_store: Arc<FsWalletStore>,
    wallet_client: Arc<RpcWalletClient>,
}

#[tokio::main]
async fn main() {
    subscribe_tracing();
    let dependencies = build_dependencies().await;
    let controller = build_controller(&dependencies);

    let server = Server::new(controller);
    server.run().await.unwrap_or_else(|e| {
        trace_error(&e);
        process::exit(1);
    });
}

fn subscribe_tracing() {
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

async fn build_dependencies() -> Dependencies {
    let wallet_store = FsWalletStore::open("wallet.db").await.unwrap_or_else(|e| {
        trace_error(&e);
        process::exit(1);
    });

    let wallet_client = RpcWalletClient::new("https://eth.llamarpc.com").unwrap_or_else(|e| {
        trace_error(&e);
        process::exit(1);
    });

    Dependencies {
        wallet_store: Arc::new(wallet_store),
        wallet_client: Arc::new(wallet_client),
    }
}

fn build_controller(dependencies: &Dependencies) -> Controller {
    let Dependencies {
        wallet_store,
        wallet_client,
    } = dependencies;

    Controller {
        wallet_list: Arc::new(wallet::ListExecutor {
            wallet_store: wallet_store.clone(),
        }),
        wallet_track: Arc::new(wallet::TrackExecutor {
            wallet_store: wallet_store.clone(),
            wallet_client: wallet_client.clone(),
        }),
        wallet_refresh: Arc::new(wallet::RefreshExecutor {
            wallet_store: wallet_store.clone(),
            wallet_client: wallet_client.clone(),
        }),
        wallet_untrack: Arc::new(wallet::UntrackExecutor {
            wallet_store: wallet_store.clone(),
        }),
    }
}

fn trace_error(error: &dyn Error) {
    let mut composed = error.to_string();

    let mut next: &dyn Error = &error;
    while let Some(source) = next.source() {
        composed.push_str(": ");
        composed.push_str(&source.to_string());
        next = source;
    }

    error!("{composed}");
}
