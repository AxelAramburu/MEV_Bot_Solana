use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use serde::{Deserialize, Serialize};
use reqwest::get;
use log::info;

pub async fn fetch_data_orca() -> Result<(), Box<dyn std::error::Error>> {
    let response = get("https://api.orca.so/allPools").await?;
    // info!("response: {:?}", response);
    // info!("response-status: {:?}", response.status().is_success());
    if response.status().is_success() {
        let json: HashMap<String, Pool> = serde_json::from_str(&response.text().await?)?;        
        // info!("json: {:?}", json);
        let mut file = File::create("src\\markets\\cache\\orca-markets.json")?;
        file.write_all(serde_json::to_string(&json)?.as_bytes())?;
        info!("Data written to 'orca-markets.json' successfully.");
    } else {
        info!("Fetch of 'orca-markets.json' not successful: {}", response.status());
    }
    Ok(())
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pool: Pool,
} 

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pool {
    pub pool_id: String,
    pub pool_account: String,
    #[serde(rename = "tokenAAmount")]
    pub token_aamount: String,
    #[serde(rename = "tokenBAmount")]
    pub token_bamount: String,
    pub pool_token_supply: String,
    pub apy: Apy,
    pub volume: Volume,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Apy {
    pub day: String,
    pub week: String,
    pub month: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub day: String,
    pub week: String,
    pub month: String,
}
