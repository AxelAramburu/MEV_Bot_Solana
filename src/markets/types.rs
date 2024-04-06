use std::{collections::HashMap, str::FromStr, sync::Arc};


#[derive(Debug)]
pub enum DexLabel {
    ORCA,
    ORCA_WHIRLPOOLS,
    RAYDIUM,
    RAYDIUM_CLMM,
}

pub impl DexLabel {
    pub fn str(&self) -> String {
        match self {
            DexLabel::ORCA => String::from("Orca"),
            DexLabel::ORCA_WHIRLPOOLS => String::from("Orca (Whirlpools)"),
            DexLabel::RAYDIUM => String::from("Raydium"),
            DexLabel::RAYDIUM_CLMM => String::from("Raydium CLMM"),
        }
    }
}

pub struct Market {
    tokenMintA: String,
    tokenVaultA: String,
    tokenMintB: String,
    tokenVaultB: String,
    dexLabel: DexLabel,
    id: String,
}

pub struct Dex {
    pairToMarkets: HashMap<String, Vec<Market>>,
    // ammCalcAddPoolMessages: AmmCalcWorkerParamMessage[];
    label: DexLabel,
}

pub impl Dex {
    pub fn new(label: DexLabel) -> Dex {
        let pairToMarkets = HashMap::new();
        Dex {
            pairToMarkets: pairToMarkets,
            label: label,
        }
    }
    
    // getAmmCalcAddPoolMessages(): AmmCalcWorkerParamMessage[] {
    //   return this.ammCalcAddPoolMessages;
    // }
    
    pub fn getMarketsForPair(mintA: String, mintB: String) -> Vec<Market> {
        let markets = self.pairToMarkets.get(toPairString(mintA, mintB));
        // if (markets === undefined) {
        // return [];
        // }   
        return markets;
    }
    
    pub fn getAllMarkets() -> Vec<Market> {
      return self.pairToMarkets;
    }

}
