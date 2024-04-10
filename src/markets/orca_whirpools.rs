use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use serde::{Deserialize, Serialize};
use reqwest::get;
use log::info;

pub async fn fetch_data_orca_whirpools() -> Result<(), Box<dyn std::error::Error>> {
    let response = get("https://api.mainnet.orca.so/v1/whirlpool/list").await?;
    // info!("response: {:?}", response);
    // info!("response-status: {:?}", response.status().is_success());
    if response.status().is_success() {
        let json: Root = serde_json::from_str(&response.text().await?)?;        
        // info!("json: {:?}", json);
        let mut file = File::create("src\\markets\\cache\\orca_whirpools-markets.json")?;
        file.write_all(serde_json::to_string(&json)?.as_bytes())?;
        info!("Data written to 'orca_whirpools-markets.json' successfully.");
    } else {
        info!("Fetch of 'orca_whirpools-markets.json' not successful: {}", response.status());
    }
    Ok(())
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub whirlpools: Vec<Whirlpool>,
    pub has_more: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Whirlpool {
    pub address: String,
    pub token_a: TokenA,
    pub token_b: TokenB,
    pub whitelisted: bool,
    pub tick_spacing: i64,
    pub price: f64,
    pub lp_fee_rate: f64,
    pub protocol_fee_rate: f64,
    pub whirlpools_config: String,
    pub modified_time_ms: Option<i64>,
    pub tvl: Option<f64>,
    pub volume: Option<Volume>,
    pub volume_denominated_a: Option<VolumeDenominatedA>,
    pub volume_denominated_b: Option<VolumeDenominatedB>,
    pub price_range: Option<PriceRange>,
    pub fee_apr: Option<FeeApr>,
    #[serde(rename = "reward0Apr")]
    pub reward0apr: Option<Reward0Apr>,
    #[serde(rename = "reward1Apr")]
    pub reward1apr: Option<Reward1Apr>,
    #[serde(rename = "reward2Apr")]
    pub reward2apr: Option<Reward2Apr>,
    pub total_apr: Option<TotalApr>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenA {
    pub mint: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i64,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
    pub coingecko_id: Option<String>,
    pub whitelisted: bool,
    pub pool_token: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenB {
    pub mint: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i64,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
    pub coingecko_id: Option<String>,
    pub whitelisted: bool,
    pub pool_token: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub day: f64,
    pub week: f64,
    pub month: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeDenominatedA {
    pub day: f64,
    pub week: f64,
    pub month: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeDenominatedB {
    pub day: f64,
    pub week: f64,
    pub month: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceRange {
    pub day: Day,
    pub week: Week,
    pub month: Month,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Day {
    pub min: f64,
    pub max: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Week {
    pub min: f64,
    pub max: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Month {
    pub min: f64,
    pub max: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeApr {
    pub day: f64,
    pub week: f64,
    pub month: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reward0Apr {
    pub day: f64,
    pub week: f64,
    pub month: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reward1Apr {
    pub day: f64,
    pub week: f64,
    pub month: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reward2Apr {
    pub day: f64,
    pub week: f64,
    pub month: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalApr {
    pub day: f64,
    pub week: f64,
    pub month: f64,
}
