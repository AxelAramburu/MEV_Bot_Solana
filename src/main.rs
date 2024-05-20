use anyhow::Result;
use log::info;
use MEV_Bot_Solana::arbitrage::strategies::run_arbitrage_strategy;
use MEV_Bot_Solana::arbitrage::streams::stream_accounts_change;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use tokio::task::JoinSet;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use MEV_Bot_Solana::common::constants::Env;
use MEV_Bot_Solana::markets::pools::load_all_pools;
use MEV_Bot_Solana::common::utils::{from_str, get_tokens_infos, setup_logger};
use MEV_Bot_Solana::arbitrage::calc_arb::calculate_arb;
use MEV_Bot_Solana::arbitrage::types::TokenInArb;

// use MEV_Bot_Solana::common::pools::{load_all_pools, Pool};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    // log4rs::init_file("logging_config.yaml", Default::default()).unwrap();
    setup_logger().unwrap();

    info!("Starting MEV_Bot_Solana");
    info!("⚠️⚠️ Liquidity is fetch to API and can be outdated on Radyium Pool");

    let env = Env::new();

    let rpc_client: RpcClient = RpcClient::new(env.rpc_url);

    let mut set: JoinSet<()> = JoinSet::new();

    info!("🏊 Launch pools fetching infos...");
    //Params is for re-fetching pools on API or not
    let dexs = load_all_pools(false).await;
    info!("🏊 {} Dexs are loaded", dexs.len());
    
    // // The first token is the base token (here SOL)
    // let tokens_to_arb: Vec<TokenInArb> = vec![
    //     TokenInArb{address: String::from("So11111111111111111111111111111111111111112"), symbol: String::from("SOL")}, // Base token here
    //     TokenInArb{address: String::from("25hAyBQfoDhfWx9ay6rarbgvWGwDdNqcHsXS3jQ3mTDJ"), symbol: String::from("MANEKI")},
    //     TokenInArb{address: String::from("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"), symbol: String::from("JUP")},
    //     TokenInArb{address: String::from("EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm"), symbol: String::from("WIF")},
    // ];
    // The first token is the base token (here SOL)
    let tokens_to_arb: Vec<TokenInArb> = vec![
        TokenInArb{address: String::from("So11111111111111111111111111111111111111112"), symbol: String::from("SOL")}, // Base token here
        TokenInArb{address: String::from("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), symbol: String::from("USDC")},
        TokenInArb{address: String::from("HwCi91TGtA5kh2WkvpWNq4KH9GV94QtDB3aCV2khiXaZ"), symbol: String::from("MCAT")},
        TokenInArb{address: String::from("vsJjNRpRvFJbzfJuqbmKHfuqqNbcXhZKpqZDrz9zxWG"), symbol: String::from("WJD")},
        TokenInArb{address: String::from("FwBixtdcmxawRFzBNeUmzhQzaFuvv6czs5wCQuLgWWsg"), symbol: String::from("CHEEPEPE")},
        TokenInArb{address: String::from("ASibGfX717oMTsjkqtJTmt8kwyXXqH3CHW4uHj9b1PUK"), symbol: String::from("1DOL")},
    ];

    let tokens_infos = get_tokens_infos(tokens_to_arb.clone()).await;
    println!("Token Infos: {:?}", tokens_infos);
    info!("📈 Launch arbitrage process...");
    // let (markets_arb, all_paths) = calculate_arb(dexs, tokens_to_arb).await;
    
    set.spawn(run_arbitrage_strategy(dexs, tokens_to_arb, tokens_infos));

    //Pseudo code
    // LOOP {
        // 1) Get all the fresh infos, with price etc 
        // 2) Compute all the paths and sort the better path if one exist
    // }
    // 3) Send transaction  

    
    while let Some(res) = set.join_next().await {
        info!("{:?}", res);
    }

    println!("End");
    Ok(())
}