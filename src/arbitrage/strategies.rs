use std::{collections::HashMap, fs::File};
use borsh::error;
use indicatif::{ProgressBar, ProgressStyle};
use rust_socketio::{asynchronous::{Client}};
use solana_sdk::pubkey::Pubkey;

use crate::{arbitrage::{
    calc_arb::{calculate_arb, get_markets_arb}, simulate::simulate_path, streams::get_fresh_accounts_states, types::{SwapPathResult, SwapRouteSimulation, VecSwapPathResult}
}, common::utils::from_str, transactions::create_transaction::{self, create_swap_transaction}};
use crate::markets::types::{Dex,Market};
use super::{simulate::simulate_path_precision, types::{SwapPath, TokenInArb, TokenInfos}};
use log::{debug, error, info};

pub async fn run_arbitrage_strategy(socket: Client, dexs: Vec<Dex>, tokens: Vec<TokenInArb>, tokens_infos: HashMap<String, TokenInfos>) {
    info!("👀 Run Arbitrage Strategies...");

    
    let markets_arb = get_markets_arb(dexs, tokens.clone()).await;

    // println!("DEBUG {:?}", fresh_markets_arb);
    // debug!("DEBUG {:?}", markets_arb.get(&"3s3CzbFzkqLvXYA93M3uHCes2nc4SiuZ11emtpDJwCht".to_string()));
    // debug!("DEBUG {:?}", fresh_markets_arb.get(&"65shmpuYmxx5p7ggNCZbyrGLCXVqbBR1ZD5aAocRBUNG".to_string()));

    // Sort markets with low liquidity
    let (sorted_markets_arb, all_paths) = calculate_arb(markets_arb.clone(), tokens.clone());

    //Get fresh account state
    let fresh_markets_arb = get_fresh_accounts_states(sorted_markets_arb.clone()).await;  
    
    // We keep route simulation result for RPC optimization
    let mut route_simulation: HashMap<Vec<u32>, Vec<SwapRouteSimulation>> = HashMap::new();
    let mut swap_paths_results: VecSwapPathResult = VecSwapPathResult{result: Vec::new()};

    let mut counter_failed_paths = 0;
    let mut counter_positive_paths = 0;
    let mut error_paths: HashMap<Vec<u32>, u8> = HashMap::new();
    
    //Progress bar
    let bar = ProgressBar::new(all_paths.len() as u64);
    bar.set_style(ProgressStyle::with_template("[{elapsed}] [{bar:140.cyan/blue}] ✅ {pos:>3}/{len:3} {msg}")
    .unwrap()
    .progress_chars("##-"));
    bar.set_message(format!("❌ Failed routes: {}/{} 💸 Positive routes: {}/{}", counter_failed_paths, bar.position(), counter_positive_paths, bar.position()));


    //Begin simulate all paths
    for (i, path) in all_paths.iter().enumerate() {     //Add this to limit iterations: .take(100)
        // println!("👀 Swap paths: {:?}", path);
        println!("👀 iiiiiiiiiiiiiiii: {}", i);

        // Verify error in previous paths to see if this path is interesting
        let key = vec![path.id_paths[0], path.id_paths[1]];
        let counter_opt = error_paths.get(&key.clone());
        match counter_opt {
            None => {},
            Some(value) => {
                if value >= &3 {
                    error!("🔴⏭️  Skip the {:?} path because previous errors", path.id_paths);
                    bar.inc(1);
                    counter_failed_paths += 1;
                    bar.set_message(format!("❌ Failed routes: {}/{} 💸 Positive routes: {}/{}", counter_failed_paths, bar.position(), counter_positive_paths, bar.position()));
                    continue;
                }
            }
        }


        // Get Pubkeys of the concerned markets
        let pubkeys: Vec<String> = path.paths.clone().iter().map(|route| route.clone().pool_address).collect();
        let markets: Vec<Market> = pubkeys.iter().filter_map(|key| fresh_markets_arb.get(key)).cloned().collect();

        let (new_route_simulation, swap_simulation_result, result_difference) = simulate_path(socket.clone(), path.clone(), markets.clone(), tokens_infos.clone(), route_simulation.clone()).await;
        
        if swap_simulation_result.len() >= path.hops as usize {
            // tokens.iter().map(|token| &token.symbol).cloned().collect::<Vec<String>>().join("-");

            let mut tokens_path = swap_simulation_result.iter().map(|swap_sim| tokens_infos.get(&swap_sim.token_in).unwrap().symbol.clone()).collect::<Vec<String>>().join("-");
            tokens_path = format!("{}-{}",tokens_path, tokens[0].symbol.clone());

            let sp_result: SwapPathResult = SwapPathResult{ 
                path_id: i as u32, 
                hops: path.hops,
                tokens_path: tokens_path, 
                route_simulations: swap_simulation_result.clone(), 
                token_in: tokens[0].address.clone(), 
                token_in_symbol: tokens[0].symbol.clone(), 
                token_out: tokens[0].address.clone(), 
                token_out_symbol: tokens[0].symbol.clone(), 
                amount_in: swap_simulation_result[0].amount_in.clone(), 
                estimated_amount_out: swap_simulation_result[swap_simulation_result.len() - 1].estimated_amount_out.clone(), 
                estimated_min_amount_out: swap_simulation_result[swap_simulation_result.len() - 1].estimated_min_amount_out.clone(), 
                result: result_difference
            };
            swap_paths_results.result.push(sp_result);

            // Reset errors if one path is good to only skip paths on 3 consecutives errors
            let key = vec![path.id_paths[0], path.id_paths[1]];
            error_paths.insert(key, 0);

            //Send interesting result in a more precise strategy
            if result_difference > 0.0 {
                counter_positive_paths += 1;
                bar.set_message(format!("❌ Failed routes: {}/{} 💸 Positive routes: {}/{}", counter_failed_paths, bar.position(), counter_positive_paths, bar.position()));

                precision_strategy(socket.clone(), path.clone(), markets, tokens.clone(), tokens_infos.clone()).await;
            }
        } else {
            counter_failed_paths += 1;

            //Code to avoid simulations of all paths on min. 3 likely consecutive failed paths, for 2 hops here
            if swap_simulation_result.len() == 0 {
                let key = vec![path.id_paths[0], path.id_paths[1]];
                let counter_opt = error_paths.get(&key.clone());
                match counter_opt {
                    None => {
                        error_paths.insert(key, 1);
                    }
                    Some(value) => {
                        error_paths.insert(key, value + 1);
                    }
                }
            }
        }

        route_simulation = new_route_simulation;

        bar.inc(1);
        bar.set_message(format!("❌ Failed routes: {}/{} 💸 Positive routes: {}/{}", counter_failed_paths, bar.position(), counter_positive_paths, bar.position()));

        if (i != 0 && i % 300 == 0) || i == all_paths.len() - 1 {
            let file_number = i / 300;
            let symbols = tokens.iter().map(|token| &token.symbol).cloned().collect::<Vec<String>>().join("-");
            let mut file = File::create(format!("results\\result_{}_{}.json", file_number, symbols)).unwrap();
            match serde_json::to_writer_pretty(&mut file, &swap_paths_results) {
                Ok(value) => {
                    info!("🥇🥇 Results writed!");
                    swap_paths_results = VecSwapPathResult{result: Vec::new()};
                }
                Err(value) => {
                    error!("Results not writed well: {:?}", value);
                }
            };
        }
        // println!("🥇🥇 Results of swaps paths");
        // for sp in swap_paths_results.result {
        //     info!("🔎 Path Id: {} // {} hop(s)", sp.path_id, sp.hops);
        //     info!("amount_in: {} {}", sp.amount_in, sp.token_in_symbol);
        //     info!("estimated_amount_out: {} {}", sp.estimated_amount_out, sp.token_out_symbol);
        // }
    }
    bar.finish();
}

pub async fn precision_strategy(socket: Client, path: SwapPath, markets: Vec<Market>, tokens: Vec<TokenInArb>, tokens_infos: HashMap<String, TokenInfos>) {

    info!("🔎🔎 Run a Precision SImulation on Path Id: {:?}", path.id_paths);

    let mut swap_paths_results: VecSwapPathResult = VecSwapPathResult{result: Vec::new()};

    let decimals = 9;
    let amounts_simulations = vec![
        5 * 10_u64.pow(decimals - 1),
        1 * 10_u64.pow(decimals),
        // 2 * 10_u64.pow(decimals),
        // 3 * 10_u64.pow(decimals),
        5 * 10_u64.pow(decimals),
        10 * 10_u64.pow(decimals),
        20 * 10_u64.pow(decimals)
    ];
    
    let mut result_amt = 0.0;
    let mut sp_to_tx: Option<SwapPathResult> = None;

    for (index, amount_in) in amounts_simulations.iter().enumerate() {
        let (swap_simulation_result, result_difference) = simulate_path_precision(amount_in.clone(), socket.clone(), path.clone(), markets.clone(), tokens_infos.clone()).await;

        if swap_simulation_result.len() >= path.hops as usize {
            let mut tokens_path = swap_simulation_result.iter().map(|swap_sim| tokens_infos.get(&swap_sim.token_in).unwrap().symbol.clone()).collect::<Vec<String>>().join("-");
            tokens_path = format!("{}-{}",tokens_path, tokens[0].symbol.clone());

            let sp_result: SwapPathResult = SwapPathResult{ 
                path_id: index as u32, 
                hops: path.hops, 
                tokens_path: tokens_path,
                route_simulations: swap_simulation_result.clone(), 
                token_in: tokens[0].address.clone(), 
                token_in_symbol: tokens[0].symbol.clone(), 
                token_out: tokens[0].address.clone(), 
                token_out_symbol: tokens[0].symbol.clone(), 
                amount_in: swap_simulation_result[0].amount_in.clone(), 
                estimated_amount_out: swap_simulation_result[swap_simulation_result.len() - 1].estimated_amount_out.clone(), 
                estimated_min_amount_out: swap_simulation_result[swap_simulation_result.len() - 1].estimated_min_amount_out.clone(), 
                result: result_difference
            };
            swap_paths_results.result.push(sp_result.clone());
            
            if result_difference > result_amt {
                result_amt = result_difference;
                println!("result_amt: {}", result_amt);
                sp_to_tx = Some(sp_result.clone());
            }
        }
    }
    if result_amt > 0.1 && sp_to_tx.is_some() {
        create_swap_transaction(create_transaction::ChainType::Devnet, create_transaction::TransactionType::CreateLUT, sp_to_tx.unwrap(), from_str("pppppppppppppppppppppppppppppppppppppppppppp").unwrap());
    }
    

}   