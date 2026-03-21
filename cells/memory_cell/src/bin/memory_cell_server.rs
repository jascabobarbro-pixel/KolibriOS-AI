//! Memory Cell Server Binary
//!
//! Runs the Memory Cell as a standalone gRPC service.

use std::net::SocketAddr;
use std::sync::Arc;

use memory_cell::{MemoryCell, grpc::MemoryCellGrpcService};
use tokio::signal;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting Memory Cell...");

    // Create the memory cell
    let cell = Arc::new(MemoryCell::new("memory-cell-0", 1024 * 1024 * 1024)); // 1GB

    // Initialize the cell
    cell.initialize().await?;
    log::info!("Memory Cell initialized");

    // Create gRPC service
    let grpc_service = MemoryCellGrpcService::new(cell.clone());

    // Configure server address
    let addr: SocketAddr = "[::1]:50051".parse()?;

    log::info!("Memory Cell gRPC server listening on {}", addr);

    // Run the server with graceful shutdown
    Server::builder()
        .add_service(
            memory_cell::grpc::proto::memory_cell::memory_cell_service_server::MemoryCellServiceServer::new(grpc_service)
        )
        .serve_with_shutdown(addr, async {
            signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
            log::info!("Shutdown signal received");
        })
        .await?;

    // Shutdown the cell
    cell.shutdown().await?;
    log::info!("Memory Cell shut down successfully");

    Ok(())
}
