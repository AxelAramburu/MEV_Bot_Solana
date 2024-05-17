use solana_sdk::pubkey::Pubkey;

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
    pub decimals: u8,
}

#[derive(Debug, Clone)]
pub struct SwapRouteSimulation {
    pub id_route: u32,
    pub pool_address: String,
    pub dex_label: DexLabel,
    pub amount_in: f64,
    pub estimated_amount_out: String,
    pub estimated_min_amount_out: String,

}
#[derive(Debug, Clone)]
pub struct SwapPathResult {
    pub id_route: u32,
    pub pool_address: String,
    pub dex_label: DexLabel,
    pub amount_in: f64,
    pub estimated_amount_out: String,
    pub estimated_min_amount_out: String,

}