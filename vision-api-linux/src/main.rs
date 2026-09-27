mod barcode;
mod error;
mod ocr;
mod routes;

use std::net::SocketAddr;

use axum::Router;
use tower_http::limit::RequestBodyLimitLayer;
use tracing_subscriber::EnvFilter;

use crate::routes::api_router;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9493_u16);

    let app = Router::new()
        .merge(api_router())
        .layer(RequestBodyLimitLayer::new(20 * 1024 * 1024));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("vision-api-linux listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    axum::serve(listener, app).await.expect("server failed");
}
