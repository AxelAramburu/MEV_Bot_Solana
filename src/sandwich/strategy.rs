use bounded_vec_deque::BoundedVecDeque;
use ethers::signers::{LocalWallet, Signer};
use ethers::{
    providers::{Middleware, Provider, Ws},
    types::{BlockNumber, H160, H256, U256, U64},
};
use log::{info, warn};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::broadcast::Sender;

// we'll update this part later, for now just import the necessary components
use crate::common::constants::{Env, WETH};
use crate::common::streams::{Event, NewBlock};
use crate::common::utils::{calculate_next_block_base_fee, to_h160};

pub async fn run_sandwich_strategy(provider: Arc<Provider<Ws>>, event_sender: Sender<Event>) {
    let mut event_receiver = event_sender.subscribe();

    loop {
        match event_receiver.recv().await {
            Ok(event) => match event {
                Event::Block(block) => {
                    info!("{:?}", block);
                }
                Event::PendingTx(mut pending_tx) => {
                    info!("{:?}", pending_tx);
                }
            },
            _ => {}
        }
    }
}