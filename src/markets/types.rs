use std::{collections::HashMap, str::FromStr, sync::Arc};
use crate::markets::utils::toPairString;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator; 
use strum_macros::EnumIter;

#[derive(Debug, Clone, EnumIter)]
pub enum DexLabel {
    ORCA,
    ORCA_WHIRLPOOLS,
    RAYDIUM,
    RAYDIUM_CLMM,
}

impl DexLabel {
    pub fn str(&self) -> String {
        match self {
            DexLabel::ORCA => String::from("Orca"),
            DexLabel::ORCA_WHIRLPOOLS => String::from("Orca (Whirlpools)"),
            DexLabel::RAYDIUM => String::from("Raydium"),
            DexLabel::RAYDIUM_CLMM => String::from("Raydium CLMM"),
        }
    }
    pub fn api_url(&self) -> String {
        match self {
            DexLabel::ORCA => String::from("https://api.orca.so/allPools"),
            DexLabel::ORCA_WHIRLPOOLS => String::from("https://api.mainnet.orca.so/v1/whirlpool/list"),
            DexLabel::RAYDIUM => String::from("https://api.raydium.io/v2/main/pairs"),
            DexLabel::RAYDIUM_CLMM => String::from("https://api.raydium.io/v2/ammV3/ammPools"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Market {
    pub tokenMintA: String,
    pub tokenVaultA: String,
    pub tokenMintB: String,
    pub tokenVaultB: String,
    pub dexLabel: DexLabel,
    pub fee: u128,
    pub id: String,
    pub account_data: Option<Vec<u8>>,
    pub liquidity: Option<u128>,
}

#[derive(Debug)]
pub struct Dex {
    pub pairToMarkets: HashMap<String, Vec<Market>>,
    // ammCalcAddPoolMessages: AmmCalcWorkerParamMessage[];
    pub label: DexLabel,
}

impl Dex {
    pub fn new(label: DexLabel) -> Self {
        let pairToMarkets = HashMap::new();
        Dex {
            pairToMarkets: pairToMarkets,
            label: label,
        }
    }
    
    // getAmmCalcAddPoolMessages(): AmmCalcWorkerParamMessage[] {
    //   return this.ammCalcAddPoolMessages;
    // }
    
    pub fn getMarketsForPair(&self, mintA: String, mintB: String) -> &Vec<Market> {
        let pair = toPairString(mintA, mintB);
        let markets = self.pairToMarkets.get(&pair).unwrap();

        return markets;
    }
    
    pub fn getAllMarkets(&self) -> Vec<&Vec<Market>> {
        let mut all_markets = Vec::new();

        for markets in self.pairToMarkets.values() {
            all_markets.push(markets);
        }
        return all_markets;
    }

}

#[derive(Debug)]
pub struct PoolItem {
    pub mintA: String,
    pub mintB: String,
    pub vaultA: String,
    pub vaultB: String,
    pub tradeFeeRate: u128
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SimulationRes {
    pub amountIn: String,
    pub estimatedAmountOut: String,
    pub estimatedMinAmountOut: Option<String>
}