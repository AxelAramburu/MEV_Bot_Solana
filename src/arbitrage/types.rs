use solana_sdk::pubkey::Pubkey;

use crate::markets::types::DexLabel;

#[derive(Debug, Clone, PartialEq)]
pub struct TokenInArb {
    pub address: String,
    pub symbol: String,
}

#[derive(Debug, Clone)]
pub struct Route {
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

}