use anyhow::Result;
use log::info;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use tokio::task::JoinSet;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use MEV_Bot_Solana::common::constants::Env;
use MEV_Bot_Solana::markets::pools::load_all_pools;
use MEV_Bot_Solana::common::utils::{setup_logger, from_str};
use MEV_Bot_Solana::arbitrage::calc_arb::calculate_arb;
use MEV_Bot_Solana::arbitrage::types::TokenInArb;

// use MEV_Bot_Solana::common::pools::{load_all_pools, Pool};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    setup_logger().unwrap();

    info!("Starting MEV_Bot_Solana");

    let env = Env::new();

    let rpc_client = RpcClient::new(env.rpc_url);
    // let ws = Ws::connect(env.wss_url.clone()).await.unwrap();
    // let provider = Arc::new(Provider::new(ws));

    // let (event_sender, _): (Sender<Event>, _) = broadcast::channel(512);

    let mut set = JoinSet::new();

    info!("🏊 Launch pools fetching infos...");
    let dexs = load_all_pools().await;
    info!("🏊 {} Dexs are loaded", dexs.len());
    
    // The first token is the base token (here SOL)
    let tokens_to_arb: Vec<TokenInArb> = vec![
        TokenInArb{address: String::from("So11111111111111111111111111111111111111112"), symbol: String::from("WSOL")}, // Base token here
        TokenInArb{address: String::from("25hAyBQfoDhfWx9ay6rarbgvWGwDdNqcHsXS3jQ3mTDJ"), symbol: String::from("MANEKI")},
        TokenInArb{address: String::from("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"), symbol: String::from("JUP")},
    ];

    info!("📈 Launch arbitrage process...");
    set.spawn(calculate_arb(dexs, tokens_to_arb));
    
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