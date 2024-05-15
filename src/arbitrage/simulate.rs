use std::collections::HashMap;

use crate::markets::{orca_whirpools::simulate_route_orca_whirpools, raydium::simulate_route_raydium, types::{DexLabel, Market}};
use super::types::{SwapPath, TokenInfos};

pub async fn simulate_path(path: SwapPath, markets: Vec<Market>, tokens_infos: HashMap<String, TokenInfos>) {
    //POTENTIALLY DROP THE PATHS IF ONE POOL HAVE NO LIQUIDITY?
    println!("NEW PATH");
    println!("Nb. Hops : {}", path.hops);
    // let AMT = 1000000000; // 1 SOL in lamport
    let mut amount_in = 1.0; // 1 SOL in lamport
    
    for route in path.paths {
        let market: Option<Market> = markets.iter().cloned().find(|market| market.id == route.pool_address);
        match route.dex {
            DexLabel::ORCA => {
                println!(" ⚠️⚠️ ONE ORCA POOL ");
            },
            DexLabel::ORCA_WHIRLPOOLS=> {
                println!("ORCA_WHIRLPOOLS - POOL");
                println!("Address: {:?}", route.pool_address);
                let amount_out = simulate_route_orca_whirpools(amount_in, route, market.unwrap(), tokens_infos.clone()).await;
                println!("Amount out: {}", amount_out);

                amount_in = amount_out.as_str().parse().expect("Bad conversion String to f64");
            },
            DexLabel::RAYDIUM => {
                println!(" ⚠️⚠️ ONE RAYDIUM POOL ");
                println!("RAYDIUM - POOL");
                println!("Address: {:?}", route.pool_address);
                let amount_out = simulate_route_raydium(amount_in, route, market.unwrap(), tokens_infos.clone()).await;
                println!("Amount out: {}", amount_out);

                amount_in = amount_out.as_str().parse().expect("Bad conversion String to f64");

            },
            DexLabel::RAYDIUM_CLMM => {
                println!(" ⚠️⚠️ ONE RAYDIUM_CLMM POOL ");
            },
        }
    }
}