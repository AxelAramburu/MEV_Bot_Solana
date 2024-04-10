use anyhow::Result;
use log::info;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use tokio::task::JoinSet;

use MEV_Bot_Solana::common::constants::Env;
use MEV_Bot_Solana::markets::pools::load_all_pools;
use MEV_Bot_Solana::common::utils::setup_logger;

// use MEV_Bot_Solana::common::pools::{load_all_pools, Pool};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    setup_logger().unwrap();

    info!("Starting MEV_Bot_Solana");
    println!("Starting MEV_Bot_Solana");

    let env = Env::new();

    // let ws = Ws::connect(env.wss_url.clone()).await.unwrap();
    // let provider = Arc::new(Provider::new(ws));

    // let (event_sender, _): (Sender<Event>, _) = broadcast::channel(512);

    let mut set = JoinSet::new();

    set.spawn(load_all_pools());
    
    // set.spawn(run_arbitrage_strategy(
    //     provider.clone(),
    //     event_sender.clone(),
    // ));
    
    while let Some(res) = set.join_next().await {
        info!("{:?}", res);
    }

    println!("End");
    Ok(())
}