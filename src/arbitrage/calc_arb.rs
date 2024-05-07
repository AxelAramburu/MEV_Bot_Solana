use std::collections::{HashMap, HashSet};

use crate::markets::types::{Dex, Market};
use crate::arbitrage::types::{TokenInArb, Route, SwapPath};

pub async fn calculate_arb(dexs: Vec<Dex>, tokens: Vec<TokenInArb>) -> (HashMap<String, Market>, Vec<SwapPath>) {

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

    let all_routes: Vec<Route> = compute_routes(markets_arb.clone());

    let all_paths: Vec<SwapPath> = generate_swap_paths(all_routes, tokens.clone());

    return (markets_arb, all_paths);
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
    // for swap_path in all_swap_paths.clone() {
        //     println!("One swap Paths - 1Hop: {:?}", swap_path);
    // }
    println!("all_swap_paths Length: {}", all_swap_paths.len());

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
    println!("all_swap_paths Length: {}", all_swap_paths.len());

    //Three hops
    // Sol -> token1 -> token2 -> token3 -> Sol
    
    // Code here...

    return all_swap_paths;
}