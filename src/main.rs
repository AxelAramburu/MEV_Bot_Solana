use std::collections::HashMap;
use std::path;

use anyhow::Result;
use futures::FutureExt;
use log::info;
use solana_sdk::pubkey::Pubkey;
use tokio::task::JoinSet;
use solana_client::rpc_client::RpcClient;
use MEV_Bot_Solana::arbitrage::strategies::{optimism_tx_strategy, run_arbitrage_strategy, sorted_interesting_path_strategy};
use MEV_Bot_Solana::markets::pools::load_all_pools;
use MEV_Bot_Solana::transactions::create_transaction::{create_ata_extendlut_transaction, ChainType, SendOrSimulate};
use MEV_Bot_Solana::{common::constants::Env, transactions::create_transaction::create_and_send_swap_transaction};
use MEV_Bot_Solana::common::utils::{from_str, get_tokens_infos, setup_logger};
use MEV_Bot_Solana::arbitrage::types::{SwapPathResult, SwapRouteSimulation, TokenInArb, TokenInfos};
use rust_socketio::{Payload, asynchronous::{Client, ClientBuilder},};


// use MEV_Bot_Solana::common::pools::{load_all_pools, Pool};

#[tokio::main]
async fn main() -> Result<()> {

    let simulation_amount = 3400000000; //3.4 SOL
    //Options
    let massive_strategie: bool = false;
    let best_strategie: bool = true;
    let optimism_strategie: bool = true;

    //Massive strat options
    let include_1hop: bool = true;
    let include_2hop: bool = true;
    let numbers_of_best_paths: usize = 2;
    let fetch_new_pools: bool = false;

    //best_strategie options
    let path_symbols: String = "SOL-TOPG".to_string();
    // let path_symbols: String = "SOL-Pepe".to_string();
    
    //Optism strategie options
    let optimism_path: String = "SOL-TOPG".to_string();

    dotenv::dotenv().ok();
    // log4rs::init_file("logging_config.yaml", Default::default()).unwrap();
    setup_logger().unwrap();

    info!("Starting MEV_Bot_Solana");
    info!("⚠️⚠️ New fresh pools fetched on METEORA and RAYDIUM are excluded because a lot of time there have very low liquidity, potentially can be used on subscribe log strategy");
    info!("⚠️⚠️ Liquidity is fetch to API and can be outdated on Radyium Pool");

    let mut set: JoinSet<()> = JoinSet::new();
    
    // // The first token is the base token (here SOL)
    let tokens_to_arb: Vec<TokenInArb> = vec![
        // TokenInArb{address: String::from("So11111111111111111111111111111111111111112"), symbol: String::from("SOL")}, // Base token here
        // TokenInArb{address: String::from("8wXtPeU6557ETkp9WHFY1n1EcU6NxDvbAggHGsMYiHsB"), symbol: String::from("GME")},
        // TokenInArb{address: String::from("EKEWAk7hfnwfR8DBb1cTayPPambqyC7pwNiYkaYQKQHp"), symbol: String::from("KITTY")},
        // TokenInArb{address: String::from("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), symbol: String::from("USDC")},
        // TokenInArb{address: String::from("AkVt31h8vgji5wF4nVbq1QmBV5wBoe8JdSoDTkDhQwEw"), symbol: String::from("WSB")},
        /////////////
        /////////////
        /////////////
        TokenInArb{address: String::from("So11111111111111111111111111111111111111112"), symbol: String::from("SOL")}, // Base token here
        TokenInArb{address: String::from("8NH3AfwkizHmbVd83SSxc2YbsFmFL4m2BeepvL6upump"), symbol: String::from("TOPG")},
        // TokenInArb{address: String::from("B5WTLaRwaUQpKk7ir1wniNB6m5o8GgMrimhKMYan2R6B"), symbol: String::from("Pepe")}, //Not the big PEPE
        // TokenInArb{address: String::from("3S8qX1MsMqRbiwKg2cQyx7nis1oHMgaCuc9c4VfvVdPN"), symbol: String::from("MOTHER")},
        // TokenInArb{address: String::from("EJ1RbQZs1r1eTAnMTVfgGekgWt6nbHjNGqcn8prxpump"), symbol: String::from("IRENE")},
        ////////
        ////////////
        // TokenInArb{address: String::from("7BgBvyjrZX1YKz4oh9mjb8ZScatkkwb8DzFx7LoiVkM3"), symbol: String::from("SLERF")},
        // TokenInArb{address: String::from("2VYVwrwSNM8WxbFdPU4KQpZUB9FWCenFFoDqvpHQ7rZE"), symbol: String::from("CUFF")},
        // TokenInArb{address: String::from("69kdRLyP5DTRkpHraaSZAQbWmAwzF9guKjZfzMXzcbAs"), symbol: String::from("USA")},
        // TokenInArb{address: String::from("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), symbol: String::from("USDC")},
        
        ///////////////
        ///////////////
        ///////////////
        // TokenInArb{address: String::from("So11111111111111111111111111111111111111112"), symbol: String::from("SOL")}, // Base token here
        // TokenInArb{address: String::from("5mbK36SZ7J19An8jFochhQS4of8g6BwUjbeCSxBSoWdp"), symbol: String::from("michi")},
        // TokenInArb{address: String::from("6D7NaB2xsLd7cauWu1wKk6KBsJohJmP2qZH9GEfVi5Ui"), symbol: String::from("SC")},
        // TokenInArb{address: String::from("EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm"), symbol: String::from("WIF")},
        // TokenInArb{address: String::from("FU1q8vJpZNUrmqsciSjp8bAKKidGsLmouB8CBdf8TKQv"), symbol: String::from("tremp")},
        // TokenInArb{address: String::from("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), symbol: String::from("USDC")},
    ];

    // let tokens_to_arb: Vec<TokenInArb> = vec![
    //     TokenInArb{address: String::from("So11111111111111111111111111111111111111112"), symbol: String::from("SOL")}, // Base token here
    //     // TokenInArb{address: String::from("5mbK36SZ7J19An8jFochhQS4of8g6BwUjbeCSxBSoWdp"), symbol: String::from("michi")},
    //     // TokenInArb{address: String::from("6D7NaB2xsLd7cauWu1wKk6KBsJohJmP2qZH9GEfVi5Ui"), symbol: String::from("SC")},
    //     // TokenInArb{address: String::from("EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm"), symbol: String::from("WIF")},
    //     // TokenInArb{address: String::from("FU1q8vJpZNUrmqsciSjp8bAKKidGsLmouB8CBdf8TKQv"), symbol: String::from("tremp")},
    //     // TokenInArb{address: String::from("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), symbol: String::from("USDC")},
    //     TokenInArb{address: String::from("8NH3AfwkizHmbVd83SSxc2YbsFmFL4m2BeepvL6upump"), symbol: String::from("TOPG")},
    // ];
    let tokens_infos: HashMap<String, TokenInfos> = get_tokens_infos(tokens_to_arb.clone()).await;

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


    if massive_strategie {
        info!("🏊 Launch pools fetching infos...");
        let dexs = load_all_pools(fetch_new_pools).await;
        info!("🏊 {} Dexs are loaded", dexs.len());
        
        
        info!("🪙🪙 Tokens Infos: {:?}", tokens_to_arb);
        info!("📈 Launch arbitrage process...");
        let result = run_arbitrage_strategy(simulation_amount, include_1hop, include_2hop, numbers_of_best_paths, dexs, tokens_to_arb.clone(), tokens_infos.clone()).await;
        let (path_for_best_strategie, swap_path_selected) = result.unwrap();

        if best_strategie {
            let _ = sorted_interesting_path_strategy(simulation_amount, path_for_best_strategie, tokens_to_arb.clone(), tokens_infos.clone()).await;
        }
    }
    
    if best_strategie {
        let path_best_strategie: String = format!("best_paths_selected/{}.json", path_symbols);
        let _ = sorted_interesting_path_strategy(simulation_amount, path_best_strategie, tokens_to_arb.clone(), tokens_infos.clone()).await;
    }
    
    if optimism_strategie {
        let _ = optimism_tx_strategy(optimism_path);
    }
    
    while let Some(res) = set.join_next().await {
        info!("{:?}", res);
    }

    println!("End");
    Ok(())
}