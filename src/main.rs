use anyhow::Result;
use ethers::providers::{Provider, Ws};
use ethers::utils::{keccak256, hex};
use log::info;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use tokio::task::JoinSet;

use MEV_Bot::common::constants::Env;
use MEV_Bot::common::streams::{start_streams, stream_new_blocks, stream_pending_transactions, Event};
use MEV_Bot::common::utils::setup_logger;
use MEV_Bot::strategies::dex_arb_base::run_arbitrage_strategy;

use MEV_Bot::common::pools::{load_all_pools, Pool};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    setup_logger().unwrap();

    info!("Starting MEV_Bot");
    println!("Starting MEV_Bot");

    let env = Env::new();

    let ws = Ws::connect(env.wss_url.clone()).await.unwrap();
    let provider = Arc::new(Provider::new(ws));

    let (event_sender, _): (Sender<Event>, _) = broadcast::channel(512);

    let mut set = JoinSet::new();

    // set.spawn(stream_new_blocks(provider.clone(), event_sender.clone()));
    
    set.spawn(run_arbitrage_strategy(
        provider.clone(),
        event_sender.clone(),
    ));
    
    while let Some(res) = set.join_next().await {
        info!("{:?}", res);
    }

    Ok(())
}