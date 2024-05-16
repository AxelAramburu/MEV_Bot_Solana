use std::collections::HashMap;

use solana_sdk::pubkey::Pubkey;

use crate::{arbitrage::{
    calc_arb::{calculate_arb, get_markets_arb}, simulate::simulate_path, streams::get_fresh_accounts_states, types::SwapPath
}, common::utils::from_Pubkey};
use crate::markets::types::{Dex, DexLabel, Market, PoolItem};
use crate::common::utils::from_str;

use super::types::{TokenInArb, TokenInfos};
use log::info;

pub async fn run_arbitrage_strategy(dexs: Vec<Dex>, tokens: Vec<TokenInArb>, tokens_infos: HashMap<String, TokenInfos>) {
    info!("👀 Run Arbitrage Strategies...");
    let markets_arb = get_markets_arb(dexs, tokens.clone()).await;
    // println!("Market Arb {:?}", markets_arb);
    let fresh_markets_arb = get_fresh_accounts_states(markets_arb.clone()).await;

    let (sorted_markets_arb, all_paths) = calculate_arb(fresh_markets_arb.clone(), tokens.clone());
    // println!("AFTER Market Arb {:?}", fresh_markets_arb);
    for path in all_paths.iter().take(5) {
        // println!("👀 Swap paths: {:?}", path);
        // Get Pubkeys of the concerned markets
        let pubkeys: Vec<String> = path.paths.clone().iter().map(|route| route.clone().pool_address).collect();
        // println!("pubkeys: {:?}", pubkeys);
        // let string = ("v59cBFTuVaeHqabC8cNsBz4Q3cgdGeon3UZEE41EQCW").to_string();
        // let field =  markets_arb.clone().get(&string);
        let markets: Vec<Market> = pubkeys.iter().filter_map(|key| fresh_markets_arb.get(key)).cloned().collect();
        // println!("markets: {:?}", markets);

        let result = simulate_path(path.clone(), markets, tokens_infos.clone()).await;
        // Take a path
        // Make a Function to simulate a swap 
        // Make the function in which we insert a path and we begin to simulate with some amount of Sol with the compute swap function and return the profit of the path
        //
    }

}