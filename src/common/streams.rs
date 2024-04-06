use ethers::{
    providers::{Middleware, Provider, Ws},
    types::*,
    utils::{keccak256, hex},
    core::types::U256
};
use serde_json::json;
use std::{error::Error, sync::Arc};
use tokio::{
    sync::broadcast::Sender, 
    task::JoinSet,
    time
};

use tokio_stream::StreamExt;

use std::time::Duration;

use eyre::Result;
use log::{info, warn};

use crate::{
    common::{tokens::Token, utils::{calculate_next_block_base_fee, u256_in_double_u128}}, 
    data::dex::Dex
};

#[derive(Default, Debug, Clone)]
pub struct NewBlock {
    pub block_number: U64,
    pub base_fee: U256,
    pub next_base_fee: U256,
}

#[derive(Debug, Clone)]
pub struct NewPendingTx {
    pub added_block: Option<U64>,
    pub tx: Transaction,
}

impl Default for NewPendingTx {
    fn default() -> Self {
        Self {
            added_block: None,
            tx: Transaction::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Block(NewBlock),
    PendingTx(NewPendingTx),
}

pub async fn start_streams(provider: Arc<Provider<Ws>>, event_sender: Sender<Event>, dex: Dex) {

    // let mut interval = time::interval(Duration::from_millis(5000));
    // loop {


        println!("startStream");
        // let mut set = JoinSet::new();

        tokio::join!(
            stream_uniswap_v2_events(provider.clone(), event_sender.clone(), dex.clone()),
            stream_uniswap_v3_events(provider.clone(), event_sender.clone(), dex.clone()),
        );
    
        // set.spawn(stream_new_blocks(provider.clone(), event_sender.clone()));
        // set.spawn(stream_uniswap_v2_events(provider.clone(), event_sender.clone(), dex.clone()));
        // set.spawn(stream_uniswap_v3_events(provider.clone(), event_sender.clone(), dex.clone()));
    
        // while let Some(res) = set.join_next().await {
        //     info!("Block number{:?}", res);
        // }
        // println!("endStream");
        
        // let block_number = stream_new_blocks(provider.clone(), event_sender.clone()).await;
        // let block_number = provider.clone().get_block_number().await.unwrap();
        // println!("new block in stream: {:?}", block_number);

        // stream_uniswap_v2_events(provider.clone(), event_sender.clone(), dex.clone()).await;
        // stream_uniswap_v3_events(provider.clone(), event_sender.clone(), dex.clone()).await;
        
        // info!("Wait 3 seconds");
        // interval.tick().await;

    // }
}

pub async fn stream_uniswap_v2_events(provider: Arc<Provider<Ws>>, event_sender: Sender<Event>, mut dex: Dex) -> Result<()> {
    // println!("ENTERV2"); 
    let block_number = provider.get_block_number().await.unwrap();

    let sync_filter =
        Filter::new().from_block(block_number - 25).event("Sync(uint112,uint112)");

        println!("Sync Filter {:?}", sync_filter);

    let mut stream = provider.subscribe_logs(&sync_filter).await?;

    while let Some(log) = stream.next().await {

        // let address = dex.pools.1.address;
        for (index, pool) in dex.pools.iter().enumerate() {
            // println!("ENTERLOOP V2");
            if pool.1.address == log.address {
                let bytes = hex::decode(log.data.to_string()).unwrap();
        
                // Split the byte array into two chunks, each representing a uint256
                let (first_chunk, second_chunk) = bytes.split_at(32);
            
                // Convert each chunk into a U256 value
                let first_u256 = U256::from_big_endian(&first_chunk);
                let second_u256 = U256::from_big_endian(&second_chunk);

                let token0: &Token = dex.tokens_map
                .iter()
                .find(|(_key, value)| pool.1.token0 == value.address)
                .map(|(key, _value)| _value)
                .unwrap();
    
                let token1: &Token = dex.tokens_map
                .iter()
                .find(|(_key, value)| pool.1.token1 == value.address)
                .map(|(key, _value)| _value)
                .unwrap();
                
                // Update reserve0 and reserve1
                dex.storage_array[[2 as usize, token0.id as usize, token1.id as usize, 2]] = first_u256.as_u128();
                dex.storage_array[[2 as usize, token1.id as usize, token0.id as usize, 3]] = second_u256.as_u128();

                info!("Uniswap V2 Pool updated: {:?}", log.address);

                // info!(
                //     " block number: {:?} address pool: {:?} ---> new reserve0 {:?}, new reverse1 {:?}",
                //     log.block_number,
                //     log.address,
                //     first_u256,
                //     second_u256
                // );
                // info!(
                //     " storage array reserve0: {:?}, reserve1: {:?}",
                //     dex.storage_array[[2 as usize, token0.id as usize, token1.id as usize, 2]],
                //     dex.storage_array[[2 as usize, token1.id as usize, token0.id as usize, 3]]
                // );
            }    
        }
        // println!("Out loop V2");
        // break;
    }
    
    // println!("out while V2");
    Ok(())
}

pub async fn stream_uniswap_v3_events(provider: Arc<Provider<Ws>>, event_sender: Sender<Event>, mut dex: Dex) -> Result<()> {
    println!("coucou");
    let block_number = provider.get_block_number().await.unwrap();
    
    let sync_filter =
        Filter::new().from_block(block_number).event("Swap(address,address,int256,int256,uint160,uint128,int24)");

    let mut stream = provider.subscribe_logs(&sync_filter).await?;

    while let Some(log) = stream.next().await {
        // let address = dex.pools.1.address;
        for (index, pool) in dex.pools.iter().enumerate() {
            
            // println!("coucou2");
            // println!("log {:?}", log);
            if pool.1.address == log.address {
                let bytes = hex::decode(log.data.to_string()).unwrap();
                
                let mut portions: Vec<&[u8]> = Vec::new();
                for chunk in bytes.chunks(32) {
                    portions.push(chunk);
                }
            
                let sqrt_price_x96 = U256::from_big_endian(&portions[2]);

                let token0: &Token = dex.tokens_map
                .iter()
                .find(|(_key, value)| pool.1.token0 == value.address)
                .map(|(key, _value)| _value)
                .unwrap();
    
                let token1: &Token = dex.tokens_map
                .iter()
                .find(|(_key, value)| pool.1.token1 == value.address)
                .map(|(key, _value)| _value)
                .unwrap();
                
                // Update sqrtPriceX96
                let (u1, u2) = u256_in_double_u128(sqrt_price_x96);
                dex.storage_array[[2 as usize, token0.id as usize, token1.id as usize, 4]] = u1;
                dex.storage_array[[2 as usize, token1.id as usize, token0.id as usize, 5]] = u2;

                info!("Uniswap V3 Pool updated: {:?}", log.address);
                // info!(
                //     " block number: {:?} address pool: {:?} ---> new sqrt_price_x96 {:?}",
                //     log.block_number,
                //     log.address,
                //     sqrt_price_x96
                //     // first_u256,
                //     // second_u256
                // );
                // info!(
                //     " storage array reserve0: {:?}, reserve1: {:?}",
                //     dex.storage_array[[2 as usize, token0.id as usize, token1.id as usize, 4]],
                //     dex.storage_array[[2 as usize, token1.id as usize, token0.id as usize, 5]]
                // );
            }    
        }
        // println!("Out loop ---------> V3");
        // break;
    }
    Ok(())
}

// A websocket connection made to get newly created blocks
pub async fn stream_new_blocks(provider: Arc<Provider<Ws>>, event_sender: Sender<Event>) {
    let stream = provider.subscribe_blocks().await.unwrap();
    let mut stream = stream.filter_map(|block| match block.number {
        Some(number) => Some(NewBlock {
            block_number: number,
            base_fee: block.base_fee_per_gas.unwrap_or_default(),
            next_base_fee: U256::from(calculate_next_block_base_fee(
                block.gas_used,
                block.gas_limit,
                block.base_fee_per_gas.unwrap_or_default(),
            )),
        }),
        None => None,
    });


    // while let Some(block) = stream.next().await {
    //     match event_sender.send(Event::Block(block)) {
    //         Ok(_) => {}
    //         Err(_) => {}
    //     }
    // }
}

// A websocket connection made to get new pending transactions
pub async fn stream_pending_transactions(provider: Arc<Provider<Ws>>, event_sender: Sender<Event>) {
    let stream = provider.subscribe_pending_txs().await.unwrap();
    let mut stream = stream.transactions_unordered(256).fuse();

    while let Some(result) = stream.next().await {
        match result {
            Ok(tx) => match event_sender.send(Event::PendingTx(NewPendingTx {
                added_block: None,
                tx,
            })) {
                Ok(_) => {}
                Err(_) => {}
            },
            Err(_) => {}
        };
    }
}