use std::collections::HashMap;

use crate::markets::{orca_whirpools::simulate_route_orca_whirpools, types::{DexLabel, Market}};

use super::types::{SwapPath, TokenInfos};

pub fn simulate_path(path: SwapPath, markets: Vec<Market>, tokens_infos: HashMap<String, TokenInfos>) {
    for route in path.paths {
        let market: Option<Market> = markets.iter().cloned().find(|market| market.id == route.pool_address);
        match route.dex {
            DexLabel::ORCA => continue,
            DexLabel::ORCA_WHIRLPOOLS=> {
                println!("Address: {:?}", route.pool_address);
                simulate_route_orca_whirpools(route, market.unwrap(), tokens_infos.clone());
                break;
            },
            DexLabel::RAYDIUM_CLMM => continue,
        }
    }
}