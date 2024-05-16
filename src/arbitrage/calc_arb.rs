use std::collections::{HashMap, HashSet};

use borsh::BorshDeserialize;
use log::info;

use crate::markets::raydium::{MarketStateLayoutV3, RaydiumPool};
use crate::markets::types::{Dex, DexLabel, Market};
use crate::arbitrage::types::{TokenInArb, Route, SwapPath};

pub async fn get_markets_arb(dexs: Vec<Dex>, tokens: Vec<TokenInArb>) -> HashMap<String, Market> {

    let mut markets_arb: HashMap<String, Market> = HashMap::new();
    let token_addresses: HashSet<String> = tokens.clone().into_iter().map(|token| token.address).collect();

    for dex in dexs {
        for (pair, market) in dex.pairToMarkets {
            //The first token is the base token (SOL)
            for market_iter in market {
                if token_addresses.contains(&market_iter.tokenMintA) && token_addresses.contains(&market_iter.tokenMintB) {
                
                    // let key = format!("{}/{:?}/{:?}", pair, market_iter.fee, dex.label);
                    // key string format example: key: "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN/So11111111111111111111111111111111111111112/400/ORCA_WHIRLPOOLS"
                    let key = format!("{}", market_iter.id);
                    // key is the address of the pool
                    markets_arb.insert(key, market_iter);
                }
            }
        }
    }

    return markets_arb;
}

pub fn calculate_arb(markets_arb: HashMap<String, Market>, tokens: Vec<TokenInArb>) -> (HashMap<String, Market>, Vec<SwapPath>) {

    //Sort valuables markets: ex: Remove low liquidity markets
    let mut sorted_markets_arb: HashMap<String, Market> = HashMap::new();
    let mut excluded_markets_arb: Vec<String> = Vec::new();

    println!("⚠️⚠️ ORCA Pool not sorted");
    println!("⚠️⚠️ ORCA_WHIRLPOOLS Pool not sorted");
    println!("⚠️⚠️ RAYDIUM_CLMM Pool not sorted");

    for (key, market) in markets_arb.clone() {
        match market.dexLabel {
            DexLabel::ORCA => {
                sorted_markets_arb.insert(key, market);
            },
            DexLabel::ORCA_WHIRLPOOLS => {
                sorted_markets_arb.insert(key, market);
            },
            DexLabel::RAYDIUM_CLMM => {
                sorted_markets_arb.insert(key, market);
            },
            DexLabel::RAYDIUM => {
                println!("⚠️⚠️ RAYDIUM Pool not sorted");
                if market.liquidity.unwrap() >= 2000 { //If liquidity more than 2000$
                    sorted_markets_arb.insert(key, market);
                } else {
                    excluded_markets_arb.push(key);
                }
            },
        }
    }
    info!("👌 Included Markets: {}", sorted_markets_arb.len());
    info!("🗑️  Excluded Markets: {}", excluded_markets_arb.len());
    let all_routes: Vec<Route> = compute_routes(sorted_markets_arb.clone());

    let all_paths: Vec<SwapPath> = generate_swap_paths(all_routes, tokens.clone());

    return (sorted_markets_arb, all_paths);
}

//Compute routes 
pub fn compute_routes(markets_arb: HashMap<String, Market>) -> Vec<Route> {
    let mut all_routes: Vec<Route> = Vec::new();
    for (key, market) in markets_arb {
        let route_0to1 = Route{dex: market.clone().dexLabel, pool_address: market.clone().id, token_0to1: true, tokenIn: market.clone().tokenMintA, tokenOut: market.clone().tokenMintB, fee: market.clone().fee};
        let route_1to0 = Route{dex: market.clone().dexLabel, pool_address: market.clone().id, token_0to1: false, tokenIn: market.clone().tokenMintB, tokenOut: market.clone().tokenMintA, fee: market.clone().fee};
        all_routes.push(route_0to1);
        all_routes.push(route_1to0);
    }

    // println!("All routes: {:?}", all_routes);
    return all_routes;
}

pub fn generate_swap_paths(all_routes: Vec<Route>, tokens: Vec<TokenInArb>) -> Vec<SwapPath> {
    // On part du postulat que les pools de même jetons, du même Dex mais avec des fees différents peuvent avoir un prix différent,
    // donc on peut créer des routes 
    let mut all_swap_paths: Vec<SwapPath> = Vec::new();

    //One hop
    // Sol -> token -> Sol
    let starting_routes: Vec<&Route> = all_routes.iter().filter(|route| route.tokenIn == tokens[0].address).collect();

    for route_x in starting_routes.clone() {
        for route_y in all_routes.clone() {
            if (route_y.tokenOut == tokens[0].address && route_x.tokenOut == route_y.tokenIn && route_x.pool_address != route_y.pool_address) {
                let paths = vec![route_x.clone(), route_y.clone()];
                all_swap_paths.push(SwapPath{hops: 1, paths: paths.clone()});
            }
        }
    }

    info!("1 Hop swap_paths length: {}", all_swap_paths.len());
    let swap_paths_1hop = all_swap_paths.len();

    //Two hops
    // Sol -> token1 -> token2 -> Sol

    for route_1 in starting_routes {
        let all_routes_2: Vec<&Route> = all_routes.iter().filter(|route| route.tokenIn == route_1.tokenOut && route_1.pool_address != route.pool_address && route.tokenOut != tokens[0].address).collect();
        for route_2 in all_routes_2 {
            let all_routes_3: Vec<&Route> = all_routes.iter().filter(|route| 
                route.tokenIn == route_2.tokenOut 
                && route_2.pool_address != route.pool_address 
                && route_1.pool_address != route.pool_address
                && route.tokenOut == tokens[0].address
            ).collect();
            if all_routes_3.len() > 0 {
                for route_3 in all_routes_3 {
                    let paths = vec![route_1.clone(), route_2.clone(), route_3.clone()];
                    all_swap_paths.push(SwapPath{hops: 2, paths: paths})
                }
            }
        }
    }
    info!("2 Hops swap_path length: {}", all_swap_paths.len() - swap_paths_1hop);

    //Three hops
    // Sol -> token1 -> token2 -> token3 -> Sol
    
    // Code here...

    return all_swap_paths;
}