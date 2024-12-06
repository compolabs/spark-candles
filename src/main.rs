use config::env::ev;
use error::Error;
use indexer::pangea::initialize_pangea_indexer;
use storage::trading_engine::{TradingEngine, TradingPairConfig};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::broadcast;
use web::server::rocket;

pub mod config;
pub mod error;
pub mod indexer;
pub mod storage;
pub mod web;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load environment variables and initialize logger
    dotenv::dotenv().ok();
    env_logger::init();

    // Load trading pair configuration
    let configs = TradingEngine::load_config("config.json")?;
    let trading_engine = Arc::new(TradingEngine::new(configs.clone()));

    // Create a broadcast channel for shutdown signals
    let (shutdown_tx, _) = broadcast::channel(1);

    // Launch Rocket server
    let port = ev("SERVER_PORT")?.parse()?;
    let rocket_task = spawn_rocket_server(port, Arc::clone(&trading_engine), shutdown_tx.subscribe());

    // Launch indexer
    let indexer_task = spawn_indexer(configs, Arc::clone(&trading_engine), shutdown_tx.subscribe());

    // Wait for Ctrl+C signal
    signal::ctrl_c().await.expect("failed to listen for Ctrl+C");
    println!("Ctrl+C received! Initiating shutdown...");

    // Notify all tasks to shut down
    drop(shutdown_tx);

    // Wait for tasks to complete
    if let Err(e) = rocket_task.await {
        eprintln!("Rocket server error: {:?}", e);
    }
    if let Err(e) = indexer_task.await {
        eprintln!("Indexer error: {:?}", e);
    }

    println!("Application has shut down gracefully.");
    Ok(())
}

fn spawn_rocket_server(
    port: u16,
    trading_engine: Arc<TradingEngine>,
    mut shutdown: broadcast::Receiver<()>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        println!("Starting Rocket server on port {}", port);
        let rocket = rocket(port, trading_engine);

        tokio::select! {
            result = rocket.launch() => {
                if let Err(e) = result {
                    eprintln!("Error launching Rocket server: {:?}", e);
                }
            }
            _ = shutdown.recv() => {
                println!("Shutdown signal received. Stopping Rocket server...");
            }
        }
    })
}

fn spawn_indexer(
    configs: Vec<TradingPairConfig>,
    trading_engine: Arc<TradingEngine>,
    mut shutdown: broadcast::Receiver<()>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(e) = initialize_pangea_indexer(configs, trading_engine, &mut shutdown).await {
            eprintln!("Indexer error: {:?}", e);
        }
    })
}
