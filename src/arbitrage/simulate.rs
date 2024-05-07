use crate::markets::{orca_whirpools::simulate_route_orca_whirpools, types::{DexLabel, Market}};

use super::types::SwapPath;

pub fn simulate_path(path: SwapPath, markets: Vec<Market>) {
    for route in path.paths {
        let market: Option<Market> = markets.iter().cloned().find(|market| market.id == route.pool_address);
        match route.dex {
            DexLabel::ORCA => continue,
            DexLabel::ORCA_WHIRLPOOLS=> {
                simulate_route_orca_whirpools(route, market.unwrap());
                break;
            },
            DexLabel::RAYDIUM_CLMM => continue,
        }
    }
}