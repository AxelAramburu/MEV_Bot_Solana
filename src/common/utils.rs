use anyhow::Result;
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use solana_program::pubkey::Pubkey;
use solana_sdk::bs58;
use core::mem;
use thiserror::Error;
// use rand::Rng;
// use std::str::FromStr;
// use std::sync::Arc;

use crate::common::constants::{PROJECT_NAME};

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