use serde::Serialize;

use crate::markets::types::DexLabel;

#[derive(Debug, Clone, PartialEq)]
pub struct TokenInArb {
    pub address: String,
    pub symbol: String,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub id: u32,
    pub dex: DexLabel,
    pub pool_address: String,
    pub token_0to1: bool,
    pub tokenIn: String,
    pub tokenOut: String,
    pub fee: u128,
}

#[derive(Debug, Clone)]
pub struct SwapPath {
    pub hops: u8,
    pub paths: Vec<Route>,
    pub id_paths: Vec<u32>,

}
#[derive(Debug, Clone)]
pub struct TokenInfos {
    pub address: String,
    pub decimals: u8,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SwapRouteSimulation {
    pub id_route: u32,
    pub pool_address: String,
    pub dex_label: DexLabel,
    pub token_0to1: bool,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: u64,
    pub estimated_amount_out: String,
    pub estimated_min_amount_out: String,

}
#[derive(Debug, Clone, Serialize)]
pub struct SwapPathResult {
    pub path_id: u32,
    pub hops: u8,
    pub tokens_path: String,
    pub route_simulations: Vec<SwapRouteSimulation>,
    pub token_in: String,
    pub token_in_symbol: String,
    pub token_out: String,
    pub token_out_symbol: String,
    pub amount_in: u64,
    pub estimated_amount_out: String,
    pub estimated_min_amount_out: String,
    pub result: f64,
}
#[derive(Debug, Clone, Serialize)]
pub struct VecSwapPathResult {
    pub result: Vec<SwapPathResult>
}