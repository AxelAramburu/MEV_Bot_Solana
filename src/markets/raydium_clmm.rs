use crate::markets::types::{Dex, DexLabel, Market};
use tokio::net::TcpStream;
use std::{fs::File, io::Read};
use std::fs;
use std::io;
use std::io::Write;
use serde::{Deserialize, Serialize};
use reqwest::get;
use log::info;


pub struct PoolItem {
    mintA: String,
    mintB: String,
    vaultA: String,
    vaultB: String,
    tradeFeeRate: u128
}

pub struct RaydiumClmmDEX {
    dex: Dex,
    pools: Vec<PoolItem>,
}
impl RaydiumClmmDEX {
    pub fn new(dex: Dex) -> Self {

        let mut pools_vec = Vec::new();
        
        let data = fs::read_to_string("src\\markets\\cache\\raydiumclmm-markets.json").expect("LogRocket: error reading file");
        let json_value: Root  = serde_json::from_str(&data).unwrap();

        for pool in json_value.data {
            let item: PoolItem = PoolItem {
                mintA: pool.mint_a.clone(),
                mintB: pool.mint_b.clone(),
                vaultA: pool.vault_a.clone(),
                vaultB: pool.vault_b.clone(),
                tradeFeeRate: pool.amm_config.trade_fee_rate.clone() as u128,
            };
            pools_vec.push(item);

            let market: Market = Market {
                tokenMintA: pool.mint_a,
                tokenVaultA: pool.vault_a,
                tokenMintB: pool.mint_b,
                tokenVaultB: pool.vault_b,
                dexLabel: DexLabel::RAYDIUM_CLMM,
                id: pool.id,
            };
        }

        Self {
            dex: dex,
            pools: pools_vec,
        }
    }
  //   constructor() {
  //     super(DexLabel.RAYDIUM_CLMM);
  //     this.pools = pools.filter((pool) => !MARKETS_TO_IGNORE.includes(pool.id));
  //     for (const pool of this.pools) {
  //       this.ammCalcAddPoolMessages.push({
  //         type: 'addPool',
  //         payload: {
  //           poolLabel: this.label,
  //           id: pool.id,
  //           feeRateBps: Math.floor(pool.ammConfig.tradeFeeRate / 100),
  //           serializableAccountInfo: toSerializableAccountInfo(
  //             initialAccountBuffers.get(pool.id),
  //           ),
  //         },
  //       });
  
  //       const market: Market = {
  //         tokenMintA: pool.mintA,
  //         tokenVaultA: pool.vaultA,
  //         tokenMintB: pool.mintB,
  //         tokenVaultB: pool.vaultB,
  //         dexLabel: this.label,
  //         id: pool.id,
  //       };
        
  //       const pairString = toPairString(pool.mintA, pool.mintB);
  //       if (this.pairToMarkets.has(pairString)) {
  //         this.pairToMarkets.get(pairString).push(market);
  //       } else {
  //         this.pairToMarkets.set(pairString, [market]);
  //       }
  //     }
  //   }
  }

pub async fn fetch_data_raydium_clmm() -> Result<(), Box<dyn std::error::Error>> {
    let response = get("https://api.raydium.io/v2/ammV3/ammPools").await?;
    // info!("response: {:?}", response);
    // info!("response-status: {:?}", response.status().is_success());
    if response.status().is_success() {
        let json: Root = serde_json::from_str(&response.text().await?)?;        
        // info!("json: {:?}", json);
        let mut file = File::create("src\\markets\\cache\\raydiumclmm-markets.json")?;
        file.write_all(serde_json::to_string(&json)?.as_bytes())?;
        info!("Data written to 'raydiumclmm-markets.json' successfully.");
    } else {
        info!("Fetch of 'raydiumclmm-markets.json'  not successful: {}", response.status());
    }
    Ok(())
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub data: Vec<PoolRaydium>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolRaydium {
    pub id: String,
    pub mint_program_id_a: String,
    pub mint_program_id_b: String,
    pub mint_a: String,
    pub mint_b: String,
    pub vault_a: String,
    pub vault_b: String,
    pub mint_decimals_a: i64,
    pub mint_decimals_b: i64,
    pub amm_config: AmmConfig,
    pub reward_infos: Vec<RewardInfo>,
    pub tvl: f64,
    pub day: Day,
    pub week: Week,
    pub month: Month,
    pub lookup_table_account: Option<String>,
    pub open_time: i64,
    pub price: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmmConfig {
    pub id: String,
    pub index: i64,
    pub protocol_fee_rate: i64,
    pub trade_fee_rate: i64,
    pub tick_spacing: i64,
    pub fund_fee_rate: i64,
    pub fund_owner: String,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardInfo {
    pub mint: String,
    pub program_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Day {
    pub volume: f64,
    pub volume_fee: f64,
    pub fee_a: f64,
    pub fee_b: f64,
    pub fee_apr: f64,
    pub reward_apr: RewardApr,
    pub apr: f64,
    pub price_min: f64,
    pub price_max: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardApr {
    #[serde(rename = "A")]
    pub a: f64,
    #[serde(rename = "B")]
    pub b: f64,
    #[serde(rename = "C")]
    pub c: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Week {
    pub volume: f64,
    pub volume_fee: f64,
    pub fee_a: f64,
    pub fee_b: f64,
    pub fee_apr: f64,
    pub reward_apr: RewardApr2,
    pub apr: f64,
    pub price_min: f64,
    pub price_max: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardApr2 {
    #[serde(rename = "A")]
    pub a: f64,
    #[serde(rename = "B")]
    pub b: f64,
    #[serde(rename = "C")]
    pub c: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Month {
    pub volume: f64,
    pub volume_fee: f64,
    pub fee_a: f64,
    pub fee_b: f64,
    pub fee_apr: f64,
    pub reward_apr: RewardApr3,
    pub apr: f64,
    pub price_min: f64,
    pub price_max: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardApr3 {
    #[serde(rename = "A")]
    pub a: f64,
    #[serde(rename = "B")]
    pub b: f64,
    #[serde(rename = "C")]
    pub c: i64,
}
