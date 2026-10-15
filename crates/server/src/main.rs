use std::{env, net::SocketAddr, path::PathBuf};

use au2_rust_server::{NoteStore, app};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "au2_rust_server=info,tower_http=info".into()),
        )
        .init();

    let addr: SocketAddr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".into())
        .parse()
        .expect("BIND_ADDR must be a socket address like 127.0.0.1:3000");

    // Serve the built frontend when it exists. In development Vite serves
    // the app and proxies /api here instead.
    let static_dir = PathBuf::from(env::var("STATIC_DIR").unwrap_or_else(|_| "web/dist".into()));
    let static_dir = static_dir
        .join("index.html")
        .is_file()
        .then_some(static_dir);

    match &static_dir {
        Some(dir) => tracing::info!("serving frontend from {}", dir.display()),
        None => tracing::info!("no built frontend found, serving the API only"),
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{}", listener.local_addr()?);

    axum::serve(listener, app(NoteStore::default(), static_dir.as_deref()))
        .with_graceful_shutdown(shutdown_signal())
        .await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to listen for SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
