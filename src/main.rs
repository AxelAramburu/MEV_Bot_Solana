use anyhow::Result;
use futures::FutureExt;
use log::info;
use MEV_Bot_Solana::arbitrage::strategies::run_arbitrage_strategy;
use std::collections::HashMap;
use tokio::task::JoinSet;
use solana_client::rpc_client::RpcClient;
use MEV_Bot_Solana::common::constants::Env;
use MEV_Bot_Solana::markets::pools::load_all_pools;
use MEV_Bot_Solana::common::utils::{get_tokens_infos, setup_logger};
use MEV_Bot_Solana::arbitrage::types::{TokenInArb, TokenInfos};

use rust_socketio::{Payload, asynchronous::{Client, ClientBuilder},};


// use MEV_Bot_Solana::common::pools::{load_all_pools, Pool};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    // log4rs::init_file("logging_config.yaml", Default::default()).unwrap();
    setup_logger().unwrap();

    info!("Starting MEV_Bot_Solana");
    info!("⚠️⚠️ New fresh pools fetched on METEORA and RAYDIUM are excluded because a lot of time there have very low liquidity, potentially can be used on subscribe log strategy");
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
    // let tokens_to_arb: Vec<TokenInArb> = vec![
    //     TokenInArb{address: String::from("So11111111111111111111111111111111111111112"), symbol: String::from("SOL")}, // Base token here
    //     TokenInArb{address: String::from("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), symbol: String::from("USDC")},
    //     TokenInArb{address: String::from("FePbYijSZfdHvswUhfBqztJ7kzUs5AEBMDi71xQhTtWC"), symbol: String::from("kiki")},
    //     TokenInArb{address: String::from("8vCAUbxejdtaxn6jnX5uaQTyTZLmXALg9u1bvFCAjtx7"), symbol: String::from("ZACK")},
    // ];
    let tokens_to_arb: Vec<TokenInArb> = vec![
        TokenInArb{address: String::from("So11111111111111111111111111111111111111112"), symbol: String::from("SOL")}, // Base token here
        // TokenInArb{address: String::from("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), symbol: String::from("USDC")},
        TokenInArb{address: String::from("9LAjk5F4rFetELE4CygcBbZ5hYc2QhRrbJjfm5Q26jWM"), symbol: String::from("DAVIDO")},
        // TokenInArb{address: String::from("GoxLaNFQiqnV97p7aRGP4ghvLZ4GwJN9NUNPpozvJZCV"), symbol: String::from("OSAK")},
    ];

    let tokens_infos: HashMap<String, TokenInfos> = get_tokens_infos(tokens_to_arb.clone()).await;

    info!("🪙🪙 Tokens Infos: {:?}", tokens_to_arb);
    info!("📈 Launch arbitrage process...");
    // let (markets_arb, all_paths) = calculate_arb(dexs, tokens_to_arb).await;
    
    info!("Open Socket IO channel...");
    let env = Env::new();
    
    let callback = |payload: Payload, socket: Client| {
        async move {
            match payload {
                Payload::String(data) => println!("Received: {}", data),
                Payload::Binary(bin_data) => println!("Received bytes: {:#?}", bin_data),
                Payload::Text(data) => println!("Received Text: {:?}", data),
            }
        }
        .boxed()
    };
    
    let mut socket = ClientBuilder::new("http://localhost:3000")
        .namespace("/")
        .on("connection", callback)
        .on("error", |err, _| {
            async move { eprintln!("Error: {:#?}", err) }.boxed()
        })
        .on("orca_quote", callback)
        .on("orca_quote_res", callback)
        .connect()
        .await
        .expect("Connection failed");
    
    // let json_payload = json!({
    //     "poolId": "HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ",
    //     "tokenInKey": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    //     "tokenInDecimals": "6",
    //     "tokenOutKey": "So11111111111111111111111111111111111111112",
    //     "tokenOutDecimals": "9",
    //     "tickSpacing": "64",
    //     "amountIn": "929",
    // });
    
    // socket
    // .emit("orca_quote", json_payload.clone())
    // .await
    // .expect("Server unreachable");
    
    set.spawn(run_arbitrage_strategy(socket, dexs, tokens_to_arb, tokens_infos));

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