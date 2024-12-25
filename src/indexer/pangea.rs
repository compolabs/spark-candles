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
use tokio::time::{interval, sleep};

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

/// Fetch historical data for a contract.
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
    let batch_size = 10_000;
    let mut last_processed_block = contract_start_block;

    let target_latest_block = get_latest_block(fuel_chain).await?;
    info!("Target last block for processing: {}", target_latest_block);

    while last_processed_block < target_latest_block {
        let to_block = (last_processed_block + batch_size).min(target_latest_block);

        let request_batch = GetSparkOrderRequest {
            from_block: Bound::Exact(last_processed_block),
            to_block: Bound::Exact(to_block),
            market_id__in: HashSet::from([contract_h256]),
            chains: HashSet::from([fuel_chain]),
            ..Default::default()
        };

        let stream_batch = client
            .get_fuel_spark_orders_by_format(request_batch, Format::JsonStream, false)
            .await
            .expect("Failed to get fuel spark orders batch");

        pangea_client::futures::pin_mut!(stream_batch);

        while let Some(data) = stream_batch.next().await {
            match data {
                Ok(data) => {
                    let data = String::from_utf8(data)?;
                    let order: PangeaOrderEvent = serde_json::from_str(&data)?;
                    handle_order_event(candle_store.clone(), order, symbol.clone()).await;
                }
                Err(e) => {
                    error!("Error in historical orders stream: {}", e);
                    break;
                }
            }
        }

        last_processed_block = to_block;
        info!("Processed events up to block {}.", last_processed_block);
    }
    Ok(last_processed_block)
}

/// Listen for new events (deltas).
async fn listen_for_new_deltas(
    client: &Client<WsProvider>,
    candle_store: &Arc<CandleStore>,
    mut last_processed_block: i64,
    contract_h256: H256,
    symbol: String,
) -> Result<(), Error> {
    let mut retry_delay = Duration::from_secs(1);
    let reconnect_interval = Duration::from_secs(10 * 60);
    let mut reconnect_timer = interval(reconnect_interval);

    loop {
        tokio::select! {
            _ = reconnect_timer.tick() => {
                info!("Refreshing connection...");
                let fuel_chain = match ev("CHAIN")?.as_str() {
                    "FUEL" => ChainId::FUEL,
                    _ => ChainId::FUELTESTNET,
                };
                let latest_block = get_latest_block(fuel_chain).await?;
                let buffer_blocks = 10;
                last_processed_block = latest_block.saturating_sub(buffer_blocks);
                info!("Updated last_processed_block to {}", last_processed_block);
            },
            result = async {
                let fuel_chain = match ev("CHAIN")?.as_str() {
                    "FUEL" => ChainId::FUEL,
                    _ => ChainId::FUELTESTNET,
                };

                let request_deltas = GetSparkOrderRequest {
                    from_block: Bound::Exact(last_processed_block + 1),
                    to_block: Bound::Subscribe,
                    market_id__in: HashSet::from([contract_h256]),
                    chains: HashSet::from([fuel_chain]),
                    ..Default::default()
                };

                match client
                    .get_fuel_spark_orders_by_format(request_deltas, Format::JsonStream, true)
                    .await
                {
                    Ok(stream_deltas) => {
                        pangea_client::futures::pin_mut!(stream_deltas);

                        while let Some(data_result) = stream_deltas.next().await {
                            match data_result {
                                Ok(data) => {
                                    let data_str = String::from_utf8(data.to_vec())?;
                                    let order_event: PangeaOrderEvent = serde_json::from_str(&data_str)?;
                                    let event_bl_num = order_event.block_number;
                                    handle_order_event(candle_store.clone(), order_event, symbol.clone()).await;
                                    last_processed_block = event_bl_num;
                                }
                                Err(e) => {
                                    error!("Error in new orders stream: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to start stream: {}", e);
                    }
                }

                sleep(retry_delay).await;
                retry_delay = (retry_delay * 2).min(Duration::from_secs(60));
                Ok::<(), Error>(())
            } => {
                if let Err(e) = result {
                    error!("Error in listen_for_new_deltas: {:?}", e);
                }
            },
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
