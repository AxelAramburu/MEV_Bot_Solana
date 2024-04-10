// use bounded_vec_deque::BoundedVecDeque;
// use ethers::signers::{LocalWallet, Signer};
// use ethers::{
//     providers::{Middleware, Provider, Ws},
//     types::{BlockNumber, H160, H256, U256, U64},
// };
// use log::{info, warn};
// use std::{collections::HashMap, str::FromStr, sync::Arc};
// use tokio::sync::broadcast::Sender;

// use crate::common::constants::Env;
// use crate::common::tokens::{get_token_info_multi, load_all_important_tokens, load_all_tokens, Token, TokenInfo};
// use crate::common::pools::{load_all_pools, Pool};
// use crate::common::streams::{start_streams, Event, NewBlock};
// use crate::common::utils::calculate_next_block_base_fee;
// use crate::common::alert::*;
// use crate::common::execution::*;
// use crate::common::utils::to_h160;


// use crate::data::dex::*;


// pub async fn run_arbitrage_strategy(provider: Arc<Provider<Ws>>, event_sender: Sender<Event>) {
//     let env = Env::new();
//     println!("Run Arbitrage Strategy");
//     let (pools, prev_pool_id) = load_all_pools(env.wss_url.clone(), 18678595, 50000)
//         .await
//         .unwrap();
    
//     let important_tokens: Vec<H160> = vec![
//         to_h160("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"), //WETH
//         to_h160("0xdac17f958d2ee523a2206206994597c13d831ec7"), //USDT
//         // to_h160("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), //USDC
//         // to_h160("0x2260fac5e5542a773aa44fbcfedf7c193bc2c599"), //WBTC
//         to_h160("0x6982508145454ce325ddbe47a25d4ec3d2311933"), //PEPE
//         // to_h160("0xa41d2f8ee4f47d3b860a149765a7df8c3287b7f0"), //SYNC
//         to_h160("0x5de8ab7e27f6e7a1fff3e5b337584aa43961beef"), //SDEX
//         // to_h160("0x95ad61b0a150d79219dcf64e1e6cc01f0b64c4ce"), //SHIB
//         // to_h160("0x7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0"), //wstETH
//         // to_h160("0xD36306A5D6BFDe4F57b5159C6518D93f171fE755"), //rstETH
//     ];

//     let block_number = provider.clone().get_block_number().await.unwrap();
//     let tokens_map: HashMap<H160, Token> = load_all_important_tokens(&provider, block_number, important_tokens)
//         .await
//         .unwrap();
//     info!("Tokens map count: {:?}", tokens_map.len());

//     // filter pools that don't have both token0 / token1 info
//     let pools_vec: Vec<Pool> = pools
//         .into_iter()
//         .filter(|p| {
//             let token0_exists = tokens_map.contains_key(&p.token0);
//             let token1_exists = tokens_map.contains_key(&p.token1);
//             token0_exists && token1_exists
//         })
//         .collect();
//     info!("Filtered pools by tokens count: {:?}", pools_vec.len());

//     let pools_map: HashMap<H160, Pool> = pools_vec
//         .clone()
//         .into_iter()
//         .map(|p| (p.address, p))
//         .collect();

//     let block = provider.clone()
//         .get_block(BlockNumber::Latest)
//         .await
//         .unwrap()
//         .unwrap();
//     let mut new_block = NewBlock {
//         block_number: block.number.unwrap(),
//         base_fee: block.base_fee_per_gas.unwrap(),
//         next_base_fee: calculate_next_block_base_fee(
//             block.gas_used,
//             block.gas_limit,
//             block.base_fee_per_gas.unwrap(),
//         ),
//     };
//     info!("Computed new block: {:?}", new_block);

//     let trading_symbols: Vec<H160> = vec![
//         to_h160("0xdac17f958d2ee523a2206206994597c13d831ec7"), //USDT
//         to_h160("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"), //WETH
//     ];
//     let trading_symbols_map: HashMap<H160, TokenInfo> = get_token_info_multi(provider.clone(), BlockNumber::Latest, &trading_symbols)
//         .await
//         .unwrap();

//     let mut dex = Dex::new(provider.clone(), trading_symbols_map, tokens_map, pools_map, new_block.block_number);
//     dex.load().await;

//     // println!("Print: {}", dex.storage_array);


//     let alert = Alert::new();
//     let executor = Executor::new(provider.clone());

//     let bot_address = H160::from_str(&env.bot_address).unwrap();
//     let wallet = env
//         .private_key
//         .parse::<LocalWallet>()
//         .unwrap()
//         .with_chain_id(1 as u64);
//     let owner = wallet.address();

//     let mut event_receiver = event_sender.subscribe();

//     //Stream pools
//     // start_streams(provider.clone(), event_sender.clone(), dex).await;

//     println!("Strategie End");

// }
