
use std::collections::HashMap;

use anyhow::Result;
use solana_client::{
        rpc_client::RpcClient,
        rpc_config::RpcAccountInfoConfig,
    };
use solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig, UiAccountData};
use solana_pubsub_client::pubsub_client::PubsubClient;
use solana_sdk::{
        pubkey::Pubkey,
        commitment_config::{CommitmentConfig, CommitmentLevel}
    };

use crate::{
    common::{
        constants::Env,
        utils::from_str,
    }, 
    markets::{
        orca::stream_orca, 
        orca_whirpools::{stream_orca_whirpools, unpack_from_slice}, 
        raydium_clmm::stream_raydium_clmm, 
        types::{DexLabel, Market}
    }
};

//Subscribe to all acounts changes with accountSubscribe
pub async fn stream_accounts_change(accounts: HashMap<Pubkey, DexLabel>) -> Result<()> {
    // for (pubkey, dex_label) in accounts.clone() {
    //     match dex_label {
    //         DexLabel::ORCA => stream_orca(pubkey).await,
    //         DexLabel::ORCA_WHIRLPOOLS => stream_orca_whirpools(pubkey).await,
    //         DexLabel::RAYDIUM_CLMM => stream_raydium_clmm(pubkey),
    //         _ => println!("Bad DexLabel in Stream"),
    //     }

    // }

    Ok(())
}

//Get fresh data on all acounts with getMultipleAccounts
pub async fn get_fresh_accounts_states(mut accounts: HashMap<String, Market>) -> HashMap<String, Market> {
    let env = Env::new();
    let rpc_client = RpcClient::new(env.rpc_url);

    let mut markets_vec: Vec<Market> = Vec::new();
    let mut key_vec: Vec<String> = Vec::new();
    let mut pubkeys_vec: Vec<Pubkey> = Vec::new();
    for (key, market) in accounts.clone().iter() {
        markets_vec.push(market.clone());
        key_vec.push(key.clone());
        pubkeys_vec.push(from_str(market.clone().id.as_str()).unwrap());
    }

    for i in (0..pubkeys_vec.len()).step_by(100) {
        let maxLength = std::cmp::min(i + 100, pubkeys_vec.len());
        let batch = &pubkeys_vec[(i..maxLength)];
        
        let batch_results = rpc_client.get_multiple_accounts(&batch).unwrap();
        for (j, account) in batch_results.iter().enumerate() {
            let account = account.clone().unwrap();
            // println!("WhirpoolAccount: {:?}", data);
            let account_data = account.data;
            markets_vec[j].account_data = Some(account_data);
            accounts.insert(key_vec[j].clone(), markets_vec[j].clone());
        }
    }
    return accounts;
}

