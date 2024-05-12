use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use solana_program::pubkey::Pubkey;
use solana_sdk::bs58;
use core::mem;
use std::collections::HashMap;
use thiserror::Error;
// use rand::Rng;
// use std::str::FromStr;
// use std::sync::Arc;

use crate::{arbitrage::types::{TokenInArb, TokenInfos}, common::constants::{
    Env, PROJECT_NAME
}};
use solana_client::{
    rpc_client::RpcClient,
};

// Function to format our console logs
pub fn setup_logger() -> Result<()> {
    let colors = ColoredLevelConfig {
        trace: Color::Cyan,
        debug: Color::Magenta,
        info: Color::Green,
        warn: Color::Red,
        error: Color::BrightRed,
        ..ColoredLevelConfig::new()
    };

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                colors.color(record.level()),
                message
            ))
        })
        .chain(std::io::stdout())
        .level(log::LevelFilter::Error)
        .level_for(PROJECT_NAME, LevelFilter::Info)
        .apply()?;

    Ok(())
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ParsePubkeyError {
    #[error("String is the wrong size")]
    WrongSize,
    #[error("Invalid Base58 string")]
    Invalid,
}

type Err = ParsePubkeyError;

/// Maximum string length of a base58 encoded pubkey
const MAX_BASE58_LEN: usize = 44;

pub fn from_str(s: &str) -> Result<Pubkey, Err> {
    if s.len() > MAX_BASE58_LEN {
        return Err(ParsePubkeyError::WrongSize);
    }
    let pubkey_vec = bs58::decode(s)
        .into_vec()
        .map_err(|_| ParsePubkeyError::Invalid)?;
    if pubkey_vec.len() != mem::size_of::<Pubkey>() {
        Err(ParsePubkeyError::WrongSize)
    } else {
        Pubkey::try_from(pubkey_vec).map_err(|_| ParsePubkeyError::Invalid)
    }
}
pub fn from_Pubkey(pubkey: Pubkey) -> String {
    let pubkey_vec = bs58::encode(pubkey)
        .into_string();
    return pubkey_vec;
}

pub async fn get_tokens_infos(tokens: Vec<TokenInArb>) -> HashMap<String, TokenInfos> {
    let env = Env::new();
    let rpc_client = RpcClient::new(env.rpc_url);

    let mut pubkeys_str: Vec<String> = Vec::new();
    let mut pubkeys: Vec<Pubkey> = Vec::new();
    for token in tokens {
        pubkeys_str.push(token.address.clone());
        pubkeys.push(from_str(token.address.clone().as_str()).unwrap());
    }
    let batch_results = rpc_client.get_multiple_accounts(&pubkeys).unwrap();

    let mut tokens_infos: HashMap<String, TokenInfos> = HashMap::new();

    for (j, account) in batch_results.iter().enumerate() {
        let account = account.clone().unwrap();
        let mint_layout = MintLayout::try_from_slice(&account.data).unwrap();

        tokens_infos.insert(pubkeys_str[j].clone(), TokenInfos{decimals: mint_layout.decimals});
    }
    return tokens_infos;

}

#[derive(BorshDeserialize, Debug)]
pub struct MintLayout {
    pub mint_authority_option: u32,
    pub mint_authority: Pubkey,
    pub supply: u64,
    pub decimals: u8,
    pub is_initialized: bool,
    pub freeze_authority_option: u32,
    pub freeze_authority: Pubkey,
}
