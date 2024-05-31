use log::info;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcSendTransactionConfig};
use solana_sdk::{commitment_config::CommitmentConfig, instruction, signature::{read_keypair_file, Keypair}, signer::Signer};
use anchor_client::Client;

use crate::{arbitrage::types::SwapPathResult, common::{constants::Env, utils::from_str}};

use super::meteoradlmm_swap::{construct_meteora_instructions, SwapParameters};

pub async fn create_transaction(transaction_infos: SwapPathResult) {
    info!("🔄 Create transaction.... ");

    let env = Env::new();

    let payer = read_keypair_file(env.payer_keypair_path).expect("Wallet keypair file not found");

    info!("💳 Wallet {:#?}", payer.pubkey());

    let commitment_config = CommitmentConfig::confirmed();

    let transaction_config: RpcSendTransactionConfig = RpcSendTransactionConfig {
        skip_preflight: false,
        preflight_commitment: Some(commitment_config.commitment),
        encoding: None,
        max_retries: None,
        min_context_slot: None,
    };

    construct_transaction(transaction_infos, payer, transaction_config).await;
    
    // ----->   Create a program, send instructions & execute on it, I can add a revert check to avoid potentials loss

    // ----->   1) Create a transaction structure 
    //          2) Craft all the instructions, ex for 2 hops :
    //              2.1) Create swap instruction (one for Meteora, raydium, etc...)
    //              2.2) 
}

pub async fn construct_transaction(transaction_infos: SwapPathResult, payer: Keypair, transaction_config: RpcSendTransactionConfig) {
    let swap_params: SwapParameters = SwapParameters{
        lb_pair: from_str(transaction_infos.route_simulations[0].pool_address.as_str()).unwrap(),
        amount_in: transaction_infos.route_simulations[0].amount_in,
        swap_for_y: transaction_infos.route_simulations[0].token_0to1,
    };
    let (compute_budget_ix, accounts, remaining_accounts, ix) = construct_meteora_instructions(transaction_config, payer, swap_params, transaction_infos.estimated_min_amount_out.parse().unwrap()).await;
}
