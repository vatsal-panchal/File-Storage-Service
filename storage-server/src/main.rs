mod engine;
mod error;
mod handlers;
mod models;

use axum::{
    routing::{delete, get, post},
    Router,
};
use engine::StorageEngine;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = std::env::var("STORAGE_DIR").map(PathBuf::from).unwrap_or_else(|_| {
        let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        base.join("data")
    });

    let engine = Arc::new(StorageEngine::new(data_dir)?);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/files", get(handlers::list_files))
        .route("/files/upload", post(handlers::upload_file))
        .route("/files/:id", get(handlers::download_file))
        .route("/files/:id/info", get(handlers::get_file_info))
        .route("/files/:id/exists", get(handlers::check_file_exists))
        .route("/files/:id", delete(handlers::delete_file))
        .layer(cors)
        .with_state(engine);

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    println!("File Storage Service running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
