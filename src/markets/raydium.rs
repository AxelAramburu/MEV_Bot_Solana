use crate::arbitrage::types::{Route, TokenInfos};
use crate::common::debug::print_json_segment;
use crate::markets::types::{Dex, DexLabel, Market, PoolItem};
use crate::markets::utils::toPairString;
use crate::common::constants::Env;

use borsh::BorshDeserialize;
use tokio::net::TcpStream;
use std::collections::HashMap;
use std::{fs::File, io::Read};
use std::fs;
use serde::{Deserialize, Deserializer, de, Serialize};
use serde_json::Value;
use reqwest::get;
use std::io::{BufWriter, Write};
use futures::StreamExt;
use log::info;
use solana_account_decoder::{UiAccountData, UiAccountEncoding};
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_pubsub_client::pubsub_client::PubsubClient;
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcAccountInfoConfig;

#[derive(Debug)]
pub struct RaydiumDEX {
    pub dex: Dex,
    pub pools: Vec<PoolItem>,
}
impl RaydiumDEX {
    pub fn new(mut dex: Dex) -> Self {

        let mut pools_vec = Vec::new();
        
        let data = fs::read_to_string("src\\markets\\cache\\raydium-markets.json").expect("LogRocket: error reading file");
        let json_value: Root  = serde_json::from_str(&data).unwrap();

        for pool in json_value.clone() {
            let item: PoolItem = PoolItem {
                mintA: pool.base_mint.clone(),
                mintB: pool.quote_mint.clone(),
                vaultA: pool.base_mint.clone(),
                vaultB: pool.quote_mint.clone(),
                tradeFeeRate: pool.volume7d.clone() as u128,
            };
            pools_vec.push(item);

            let market: Market = Market {
                tokenMintA: pool.base_mint.clone(),
                tokenVaultA: pool.base_mint.clone(),
                tokenMintB: pool.quote_mint.clone(),
                tokenVaultB: pool.quote_mint.clone(),
                dexLabel: DexLabel::RAYDIUM,
                fee: pool.volume7d.clone() as u128,
                id: pool.market.clone(),
                account_data: None,
            };

            let pair_string = toPairString(pool.base_mint, pool.quote_mint);
            if dex.pairToMarkets.contains_key(&pair_string.clone()) {
                let vec_market = dex.pairToMarkets.get_mut(&pair_string).unwrap();
                vec_market.push(market);
            } else {
                dex.pairToMarkets.insert(pair_string, vec![market]);
            }
        }

        info!("Raydium : {} pools founded", json_value.len());
        Self {
            dex: dex,
            pools: pools_vec,
        }
    }
  }

// pub async fn fetch_data_raydium() -> Result<(), Box<dyn std::error::Error>> {
//     let response = get("https://api.raydium.io/v2/main/pairs").await?;
//     // info!("response: {:?}", response);
//     // info!("response-status: {:?}", response.status().is_success());
//     if response.status().is_success() {
//         let json: Root = serde_json::from_str(&response.text().await?)?;        
//         // let json = &response.text().await?;        
//         info!("json: {:?}", json);
//         let mut file = File::create("src\\markets\\cache\\raydium-markets.json")?;
//         file.write_all(serde_json::to_string(&json)?.as_bytes())?;
//         info!("Data written to 'raydium-markets.json' successfully.");
//     } else {
//         info!("Fetch of 'raydium-markets.json' not successful: {}", response.status());
//     }
//     Ok(())
// }
pub async fn fetch_data_raydium() -> Result<(), Box<dyn std::error::Error>> {
    let response = get("https://api.raydium.io/v2/main/pairs").await?;
    // info!("response: {:?}", response);
    // info!("response-status: {:?}", response.status().is_success());
    if response.status().is_success() {
        let data = response.text().await?;
        
        match serde_json::from_str::<Root>(&data) {
            Ok(json) => {
                let file = File::create("src/markets/cache/raydium-markets.json")?;
                let mut writer = BufWriter::new(file);
                writer.write_all(serde_json::to_string(&json)?.as_bytes())?;
                writer.flush()?;
                info!("Data written to 'raydium-markets.json' successfully.");
            }
            Err(e) => {
                eprintln!("Failed to deserialize JSON: {:?}", e);
                // Optionally, save the raw JSON data to inspect it manually
                // let mut raw_file = File::create("src/markets/cache/raydium-markets-raw.json")?;
                let result = print_json_segment("src/markets/cache/raydium-markets.json", 21174733 - 1000 as u64, 2000);
                // raw_file.write_all(data.as_bytes())?;
                // info!("Raw data written to 'raydium-markets-raw.json' for inspection.");
            }
        }
    } else {
        info!("Fetch of 'raydium-markets.json' not successful: {}", response.status());
    }
    Ok(())
}

pub async fn stream_raydium(account: Pubkey) -> Result<()> {
    let env = Env::new();
    let url = env.wss_rpc_url.as_str();
    let (mut account_subscription_client, account_subscription_receiver) =
    PubsubClient::account_subscribe(
        url,
        &account,
        Some(RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::JsonParsed),
            data_slice: None,
            commitment: Some(CommitmentConfig::confirmed()),
            min_context_slot: None,
        }),
    )?;

    loop {
        match account_subscription_receiver.recv() {
            Ok(response) => {
                let data = response.value.data;
                let bytes_slice = UiAccountData::decode(&data).unwrap();
                println!("account subscription data response: {:?}", data);
                // let account_data = unpack_from_slice(bytes_slice.as_slice());
                // println!("Raydium CLMM Pool updated: {:?}", account);
                // println!("Data: {:?}", account_data.unwrap());

            }
            Err(e) => {
                println!("account subscription error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}

// Simulate one route 
pub async fn simulate_route_raydium(amount_in: f64, route: Route, market: Market, tokens_infos: HashMap<String, TokenInfos>) -> String {
    // I want to get the data of the market i'm interested in this route
    let raydium_data = RaydiumPool::try_from_slice(&market.account_data.unwrap()).unwrap();
    let mut decimals_0: u8 = 0;
    let mut decimals_1: u8 = 0;
    
    // if route.token_0to1 == true {
        decimals_0 = tokens_infos.get(&market.tokenMintA).unwrap().decimals;
        decimals_1 = tokens_infos.get(&market.tokenMintB).unwrap().decimals;
    // } else {
    //     decimals_0 = tokens_infos.get(&market.tokenMintB).unwrap().decimals;
    //     decimals_1 = tokens_infos.get(&market.tokenMintA).unwrap().decimals; 
    // }

    //Get price
    // let price = from_x64_orca_wp(whirpool_data.sqrt_price, decimals_0 as f64, decimals_1 as f64);
    // println!("Price: {:?}", price);

    // Simulate a swap
    let env = Env::new();
    let domain = env.simulator_url;

    let params = format!(
        "tokenInKey={}&tokenInDecimals={}&tokenOutKey={}&tokenOutDecimals={}&tickSpacing={}&amountIn={}",
        raydium_data.base_mint,
        decimals_0,
        raydium_data.quote_mint,
        decimals_1,
        raydium_data.tick_spacing,
        amount_in,
    );
    let req_url = format!("{}orca_quote?{}", domain, params);
    // println!("req_url: {:?}", req_url);

    let mut res = reqwest::get(req_url).await.expect("Error in request to simulator");

    let json_value = res.json::<SimulationRes>().await;
    match json_value {
        Ok(value) => {
            println!("estimatedAmountIn: {:?}", value.estimatedAmountIn);
            println!("estimatedAmountOut: {:?}", value.estimatedAmountOut);
            return value.estimatedAmountOut;
        }
        Err(value) => { format!("value{:?}", value) }
    }

}

fn de_rating<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse().map_err(de::Error::custom)?,
        Value::Number(num) => num.as_f64().ok_or(de::Error::custom("Invalid number"))? as f64,
        Value::Null => 0.0,
        _ => return Err(de::Error::custom("wrong type"))
    })
}

pub type Root = Vec<RaydiumPool>;

#[derive(Default, BorshDeserialize, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaydiumPool {
    pub name: String,
    pub amm_id: String,
    pub lp_mint: String,
    pub base_mint: String,
    pub quote_mint: String,
    pub market: String,
    #[serde(deserialize_with = "de_rating")]
    pub liquidity: f64,
    #[serde(deserialize_with = "de_rating")]
    pub volume24h: f64,
    #[serde(deserialize_with = "de_rating")]
    pub volume24h_quote: f64,
    #[serde(deserialize_with = "de_rating")]
    pub fee24h: f64,
    #[serde(deserialize_with = "de_rating")]
    pub fee24h_quote: f64,
    #[serde(deserialize_with = "de_rating")]
    pub volume7d: f64,
    #[serde(deserialize_with = "de_rating")]
    pub volume7d_quote: f64,
    #[serde(deserialize_with = "de_rating")]
    pub fee7d: f64,
    #[serde(deserialize_with = "de_rating")]
    pub fee7d_quote: f64,
    #[serde(deserialize_with = "de_rating")]
    pub volume30d: f64,
    #[serde(deserialize_with = "de_rating")]
    pub volume30d_quote: f64,
    #[serde(deserialize_with = "de_rating")]
    pub fee30d: f64,
    #[serde(deserialize_with = "de_rating")]
    pub fee30d_quote: f64,
    #[serde(deserialize_with = "de_rating")]
    pub price: f64,
    #[serde(deserialize_with = "de_rating")]
    pub lp_price: f64,
    #[serde(deserialize_with = "de_rating")]
    pub token_amount_coin: f64,
    #[serde(deserialize_with = "de_rating")]
    pub token_amount_pc: f64,
    #[serde(deserialize_with = "de_rating")]
    pub token_amount_lp: f64,
    #[serde(deserialize_with = "de_rating")]
    pub apr24h: f64,
    #[serde(deserialize_with = "de_rating")]
    pub apr7d: f64,
    #[serde(deserialize_with = "de_rating")]
    pub apr30d: f64,
}