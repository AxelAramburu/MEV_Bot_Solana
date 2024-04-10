use crate::markets::types::{Dex, DexLabel};
use crate::markets::raydium_clmm::{RaydiumClmmDEX, fetch_data_raydium_clmm};
use crate::markets::orca::{fetch_data_orca};
use crate::markets::orca_whirpools::{fetch_data_orca_whirpools};
use strum::IntoEnumIterator;
use std::fs::File;
use std::io::Write;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use reqwest::get;
use log::info;

pub async fn load_all_pools() {
    println!("coucou");
    
    fetch_data_raydium_clmm().await;
    fetch_data_orca().await;
    fetch_data_orca_whirpools().await;

    let dex1 = Dex::new(DexLabel::RAYDIUM_CLMM);
    let dex1 = RaydiumClmmDEX::new(dex1);

}

