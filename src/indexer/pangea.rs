use ethers_core::types::H256;
use fuels::accounts::provider::Provider;
use log::{error, info};
use pangea_client::{
    futures::StreamExt, provider::FuelProvider, query::Bound, requests::fuel::GetSparkOrderRequest,
    ClientBuilder, Format, WsProvider,
};
use pangea_client::{ChainId, Client};
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::sleep;

use crate::config::env::ev;
use crate::error::Error;
use crate::indexer::order_event_handler::handle_order_event;
use crate::indexer::order_event_handler::PangeaOrderEvent;
use crate::storage::candles::CandleStore;
use crate::storage::trading_engine::{TradingEngine, TradingPairConfig};

pub async fn initialize_pangea_indexer(
    configs: Vec<TradingPairConfig>,
    trading_engine: Arc<TradingEngine>,
    shutdown: &mut broadcast::Receiver<()>,
) -> Result<(), Error> {
    let mut tasks = Vec::new();

    for config in configs {
        let store = match trading_engine.get_store(&config.symbol) {
            Some(s) => s,
            None => {
                log::error!("No CandleStore found for symbol {}", config.symbol);
                continue;
            }
        };

        tasks.push(tokio::spawn(process_events_for_pair(config, store)));
    }

    tokio::select! {
        _ = shutdown.recv() => {
            log::info!("Shutdown signal received in indexer.");
        }
        _ = futures::future::join_all(tasks) => {
            log::info!("All indexer tasks completed.");
        }
    }

    Ok(())
}

async fn process_events_for_pair(
    config: TradingPairConfig,
    store: Arc<CandleStore>,
) -> Result<(), Error> {
    let client = create_pangea_client().await?;

    let contract_h256 = H256::from_str(&config.contract_id)?;

    let last_processed_block = fetch_historical_data(
        &client,
        &store,
        config.start_block,
        contract_h256,
        config.symbol.clone(),
    )
    .await?;

    log::info!(
        "Completed historical data fetch for {}. Last processed block: {}",
        config.symbol,
        last_processed_block
    );

    listen_for_new_deltas(
        &client,
        &store,
        last_processed_block,
        contract_h256,
        config.symbol,
    )
    .await?;

    Ok(())
}

/// Create a Pangea WebSocket client.
async fn create_pangea_client() -> Result<Client<WsProvider>, Error> {
    let username = ev("PANGEA_USERNAME")?;
    let password = ev("PANGEA_PASSWORD")?;
    let url = ev("PANGEA_URL")?;

    let client = ClientBuilder::default()
        .endpoint(&url)
        .credential(username, password)
        .build::<WsProvider>()
        .await?;

    info!("Pangea WebSocket client connected.");
    Ok(client)
}

/// Fetch historical data for a contract without batching.
async fn fetch_historical_data(
    client: &Client<WsProvider>,
    candle_store: &Arc<CandleStore>,
    contract_start_block: i64,
    contract_h256: H256,
    symbol: String,
) -> Result<i64, Error> {
    let fuel_chain = match ev("CHAIN")?.as_str() {
        "FUEL" => ChainId::FUEL,
        _ => ChainId::FUELTESTNET,
    };

    let target_latest_block = get_latest_block(fuel_chain).await?;
    info!(
        "Fetching historical data from block {} to {}",
        contract_start_block, target_latest_block
    );

    let request = GetSparkOrderRequest {
        from_block: Bound::Exact(contract_start_block),
        to_block: Bound::Exact(target_latest_block),
        market_id__in: HashSet::from([contract_h256]),
        chains: HashSet::from([fuel_chain]),
        ..Default::default()
    };

    let stream = client
        .get_fuel_spark_orders_by_format(request, Format::JsonStream, false)
        .await?;

    pangea_client::futures::pin_mut!(stream);

    while let Some(data) = stream.next().await {
        match data {
            Ok(data) => {
                let order: PangeaOrderEvent = serde_json::from_slice(&data)?;
                handle_order_event(candle_store.clone(), order, symbol.clone()).await;
            }
            Err(e) => {
                error!("Error processing historical data stream: {}", e);
                break;
            }
        }
    }

    Ok(target_latest_block)
}

/// Listen for new events (deltas) without unnecessary reconnects.
async fn listen_for_new_deltas(
    client: &Client<WsProvider>,
    candle_store: &Arc<CandleStore>,
    mut last_processed_block: i64,
    contract_h256: H256,
    symbol: String,
) -> Result<(), Error> {
    let mut retry_delay = Duration::from_secs(1);

    loop {
        let fuel_chain = match ev("CHAIN")?.as_str() {
            "FUEL" => ChainId::FUEL,
            _ => ChainId::FUELTESTNET,
        };

        let request = GetSparkOrderRequest {
            from_block: Bound::Exact(last_processed_block + 1),
            to_block: Bound::Subscribe,
            market_id__in: HashSet::from([contract_h256]),
            chains: HashSet::from([fuel_chain]),
            ..Default::default()
        };

        match client
            .get_fuel_spark_orders_by_format(request, Format::JsonStream, true)
            .await
        {
            Ok(stream) => {
                pangea_client::futures::pin_mut!(stream);
                retry_delay = Duration::from_secs(1); // Reset retry delay on success

                while let Some(data) = stream.next().await {
                    match data {
                        Ok(data) => {
                            let order_event: PangeaOrderEvent = serde_json::from_slice(&data)?;
                            last_processed_block = order_event.block_number;
                            handle_order_event(candle_store.clone(), order_event, symbol.clone())
                                .await;
                        }
                        Err(e) => {
                            error!("Stream error: {}", e);
                            break; // Exit the stream and reconnect
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to subscribe to new deltas: {}", e);
                sleep(retry_delay).await;
                retry_delay = (retry_delay * 2).min(Duration::from_secs(60));
            }
        }
    }
}

/// Get the latest block number for a chain.
async fn get_latest_block(chain_id: ChainId) -> Result<i64, Error> {
    let provider_url = match chain_id {
        ChainId::FUEL => "mainnet.fuel.network",
        ChainId::FUELTESTNET => "testnet.fuel.network",
        _ => return Err(Error::UnknownChainIdError),
    };
    let provider = Provider::connect(provider_url).await?;
    Ok(provider.latest_block_height().await? as i64)
}
