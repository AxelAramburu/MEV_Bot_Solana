use std::{collections::HashMap, fs::File};

use serde_json::{Result, Value};

use solana_sdk::pubkey::Pubkey;

use crate::{arbitrage::{
    calc_arb::{calculate_arb, get_markets_arb}, simulate::simulate_path, streams::get_fresh_accounts_states, types::{SwapPath, SwapPathResult, SwapRouteSimulation, VecSwapPathResult}
}, common::utils::from_Pubkey};
use crate::markets::types::{Dex, DexLabel, Market, PoolItem};
use crate::common::utils::from_str;

use super::types::{TokenInArb, TokenInfos};
use log::{info, error};

pub async fn run_arbitrage_strategy(dexs: Vec<Dex>, tokens: Vec<TokenInArb>, tokens_infos: HashMap<String, TokenInfos>) {
    info!("👀 Run Arbitrage Strategies...");
    let markets_arb = get_markets_arb(dexs, tokens.clone()).await;
    let fresh_markets_arb = get_fresh_accounts_states(markets_arb.clone()).await;
    let (sorted_markets_arb, all_paths) = calculate_arb(fresh_markets_arb.clone(), tokens.clone());

    // We keep route simulation result for RPC optimization
    let mut route_simulation: HashMap<Vec<u32>, Vec<SwapRouteSimulation>> = HashMap::new();

    let mut swap_paths_results: VecSwapPathResult = VecSwapPathResult{result: Vec::new()};
    for (i, path) in all_paths.iter().enumerate() {     //Add this to limit iterations: .take(100)
        // println!("👀 Swap paths: {:?}", path);
        // Get Pubkeys of the concerned markets
        let pubkeys: Vec<String> = path.paths.clone().iter().map(|route| route.clone().pool_address).collect();
        // println!("pubkeys: {:?}", pubkeys);
        // let string = ("v59cBFTuVaeHqabC8cNsBz4Q3cgdGeon3UZEE41EQCW").to_string();
        // let field =  markets_arb.clone().get(&string);
        let markets: Vec<Market> = pubkeys.iter().filter_map(|key| sorted_markets_arb.get(key)).cloned().collect();
        // println!("route_simulation: {:?}", route_simulation);

        let (new_route_simulation, swap_simulation_result) = simulate_path(path.clone(), markets, tokens_infos.clone(), route_simulation.clone()).await;

        if swap_simulation_result.len() >= path.hops as usize {
            let sp_result: SwapPathResult = SwapPathResult{ 
                path_id: i as u32, 
                hops: path.hops, 
                route_simulations: swap_simulation_result.clone(), 
                token_in: tokens[0].address.clone(), 
                token_in_symbol: tokens[0].symbol.clone(), 
                token_out: tokens[0].address.clone(), 
                token_out_symbol: tokens[0].symbol.clone(), 
                amount_in: swap_simulation_result[0].amount_in.clone(), 
                estimated_amount_out: swap_simulation_result[swap_simulation_result.len() - 1].estimated_amount_out.clone(), 
                estimated_min_amount_out: swap_simulation_result[swap_simulation_result.len() - 1].estimated_min_amount_out.clone(), 
                result: 0.0
            };
            swap_paths_results.result.push(sp_result);
        }

        route_simulation = new_route_simulation;

    }
    let mut file = File::create("results\\result.json").unwrap();
    match serde_json::to_writer_pretty(&mut file, &swap_paths_results) {
        Ok(value) => {
            info!("🥇🥇 Results writed!")
        }
        Err(value) => {
            error!("Results not writed well: {:?}", value);
        }
    };

    println!("🥇🥇 Results of swaps paths");
    for sp in swap_paths_results.result {
        info!("🔎 Path Id: {} // {} hop(s)", sp.path_id, sp.hops);
        info!("amount_in: {} {}", sp.amount_in, sp.token_in_symbol);
        info!("estimated_amount_out: {} {}", sp.estimated_amount_out, sp.token_out_symbol);
    }

}