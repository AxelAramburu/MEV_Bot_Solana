use crate::common::constants::Env;
use crate::markets::types::{Dex, DexLabel};
use crate::markets::raydium_clmm::{RaydiumClmmDEX, fetch_data_raydium_clmm};
use crate::markets::orca::{OrcaDex, fetch_data_orca};
use crate::markets::orca_whirpools::{OrcaDexWhirpools, fetch_data_orca_whirpools};

use strum::IntoEnumIterator;
use std::fs::File;
use std::io::Write;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use reqwest::get;
use log::info;
use solana_client::rpc_client::RpcClient;

pub async fn load_all_pools() -> Vec<Dex> {
    fetch_data_raydium_clmm().await;
    fetch_data_orca().await;
    fetch_data_orca_whirpools().await;

    let mut dex1 = Dex::new(DexLabel::RAYDIUM_CLMM);
    let dex_raydium = RaydiumClmmDEX::new(dex1);
    let mut dex2 = Dex::new(DexLabel::ORCA);
    let dex_orca = OrcaDex::new(dex2);
    let mut dex3 = Dex::new(DexLabel::ORCA_WHIRLPOOLS);
    let dex_orca_whirpools = OrcaDexWhirpools::new(dex3);
    
    // println!("random_pair {:?}", dex_raydium.dex.pairToMarkets.get("So11111111111111111111111111111111111111112/mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So"));
    
    let mut results: Vec<Dex> = Vec::new();
    results.push(dex_raydium.dex);
    results.push(dex_orca.dex);
    results.push(dex_orca_whirpools.dex);
    return results

}

