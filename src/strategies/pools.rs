use std::{collections::HashMap, mem};

use log::info;
use solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig};
use solana_client::{
    pubsub_client::PubsubClient, rpc_client::RpcClient, rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig}, rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType}
};
use solana_sdk::{commitment_config::CommitmentConfig, signer::Signer};

use crate::{
    arbitrage::types::TokenInArb, common::{
        constants::Env,
        utils::from_str,
    }, markets::{orca_whirpools::{fetch_new_orca_whirpools, unpack_from_slice, WhirlpoolAccount}, raydium::fetch_new_raydium_pools, types::{DexLabel, Market}} 
};

pub async fn get_fresh_pools(tokens: Vec<TokenInArb>) -> HashMap<String, Market> {
    let env = Env::new();
    let rpc_client = RpcClient::new_with_commitment(env.rpc_url.as_str(), CommitmentConfig::confirmed());
    let mut new_markets: HashMap<String, Market> = HashMap::new();
    let mut count_new_pools = 0;

    for (i, token) in tokens.iter().enumerate() {
        // Avoid fetch for the first token (often SOL)
        if i == 0 {
            continue;
        }
        //Orca Whirpools 
        let orca_res_tokena = fetch_new_orca_whirpools(&rpc_client, token.address.clone(), false).await;
        for orca_pool in orca_res_tokena {
            new_markets.insert(orca_pool.0.to_string(), orca_pool.1);
            count_new_pools += 1;
        }
        let orca_res_tokenb = fetch_new_orca_whirpools(&rpc_client, token.address.clone(), true).await;
        for orca_pool in orca_res_tokenb {
            new_markets.insert(orca_pool.0.to_string(), orca_pool.1);
            count_new_pools += 1;
        }
        //Raydium Markets 
        let orca_res_tokena = fetch_new_raydium_pools(&rpc_client, token.address.clone(), false).await;
        for orca_pool in orca_res_tokena {
            new_markets.insert(orca_pool.0.to_string(), orca_pool.1);
            count_new_pools += 1;
        }
        let orca_res_tokenb = fetch_new_raydium_pools(&rpc_client, token.address.clone(), true).await;
        for orca_pool in orca_res_tokenb {
            new_markets.insert(orca_pool.0.to_string(), orca_pool.1);
            count_new_pools += 1;
        }
    }
    info!("👀 {} new markets founded !", count_new_pools);
    return new_markets;
}