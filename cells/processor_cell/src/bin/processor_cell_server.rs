//! Processor Cell Server Binary

use std::net::SocketAddr;
use std::sync::Arc;

use processor_cell::{ProcessorCell, grpc::ProcessorCellGrpcService};
use tokio::signal;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting Processor Cell...");

    let cell = Arc::new(ProcessorCell::new("processor-cell-0"));

    cell.initialize().await?;
    log::info!("Processor Cell initialized");

    let grpc_service = ProcessorCellGrpcService::new(cell.clone());

    let addr: SocketAddr = "[::1]:50052".parse()?;

    log::info!("Processor Cell gRPC server listening on {}", addr);

    Server::builder()
        .add_service(
            processor_cell::grpc::proto::processor_cell::processor_cell_service_server::ProcessorCellServiceServer::new(grpc_service)
        )
        .serve_with_shutdown(addr, async {
            signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
            log::info!("Shutdown signal received");
        })
        .await?;

    cell.shutdown().await?;
    log::info!("Processor Cell shut down successfully");

    Ok(())
}
