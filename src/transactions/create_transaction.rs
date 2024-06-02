use log::info;
use solana_client::{rpc_client::RpcClient, rpc_config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig}};
use solana_sdk::{commitment_config::CommitmentConfig, compute_budget::ComputeBudgetInstruction, instruction::{self, Instruction}, signature::{read_keypair_file, Keypair}, signer::Signer};
use solana_sdk::{pubkey::Pubkey, transaction::Transaction};

use crate::{arbitrage::types::SwapPathResult, common::{constants::Env, utils::from_str}, markets::types::DexLabel};

// use super::{meteoradlmm_swap::{SwapParametersMeteora}};
use super::{meteoradlmm_swap::{construct_meteora_instructions, SwapParametersMeteora}, raydium_swap::{construct_raydium_instructions, SwapParametersRaydium}};

pub async fn create_transaction(transaction_infos: SwapPathResult) {
    info!("🔄 Create transaction.... ");

    // let wallet = Keypair::from_base58_string(
    //     "",
    // );

    // println!("Keypair: {:?}", wallet);

    let env = Env::new();
    let rpc_client: RpcClient = RpcClient::new(env.rpc_url);
    // let rpc_client: RpcClient = RpcClient::new(env.rpc_url);

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
    
    let swap_instructions = construct_transaction(transaction_infos, transaction_config).await;
    
    let txn = Transaction::new_signed_with_payer(
        &swap_instructions,
        Some(&payer.pubkey()),
        &vec![&payer],
        rpc_client.get_latest_blockhash().expect("Error in get latest blockhash"),
    );

    let config = RpcSimulateTransactionConfig {
        sig_verify: true,
        .. RpcSimulateTransactionConfig::default()
    };

    println!("Txn: {:?}", txn);

    let result = rpc_client.simulate_transaction_with_config(&txn, config).unwrap().value;

    info!("Simulate Tx Logs: {:#?}", result.logs);

    // ----->   Create a program, send instructions & execute on it, I can add a revert check to avoid potentials loss

    // ----->   1) Create a transaction structure 
    //          2) Craft all the instructions, ex for 2 hops :
    //              2.1) Create swap instruction (one for Meteora, raydium, etc...)
    //              2.2) 
}

pub async fn construct_transaction(transaction_infos: SwapPathResult, transaction_config: RpcSendTransactionConfig) -> Vec<Instruction> {
    
    let mut swap_instructions: Vec<Instruction> = Vec::new();
    let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(1_400_000);
    swap_instructions.push(compute_budget_ix);
    
    for (i, route_sim) in transaction_infos.route_simulations.clone().iter().enumerate() {
        match route_sim.dex_label {
            DexLabel::METEORA => {
                let swap_params: SwapParametersMeteora = SwapParametersMeteora{
                    lb_pair: from_str(transaction_infos.route_simulations[i].pool_address.as_str()).unwrap(),
                    amount_in: transaction_infos.route_simulations[i].amount_in,
                    swap_for_y: transaction_infos.route_simulations[i].token_0to1,
                    input_token: from_str(transaction_infos.route_simulations[i].token_in.as_str()).unwrap(),
                    output_token: from_str(transaction_infos.route_simulations[i].token_out.as_str()).unwrap(),
                    minimum_amount_out: transaction_infos.route_simulations[i].estimated_amount_out.parse().unwrap()
                };
                let result = construct_meteora_instructions(swap_params.clone()).await;
                for instruction in result {
                    swap_instructions.push(instruction);
                }
            }
            DexLabel::RAYDIUM => {
                let swap_params: SwapParametersRaydium = SwapParametersRaydium{
                    pool: from_str(transaction_infos.route_simulations[i].pool_address.as_str()).unwrap(),
                    input_token_mint: from_str(route_sim.token_in.as_str()).unwrap(),
                    output_token_mint: from_str(route_sim.token_out.as_str()).unwrap(),
                    amount_in: transaction_infos.route_simulations[i].amount_in,
                    swap_for_y: transaction_infos.route_simulations[i].token_0to1,
                    min_amount_out: transaction_infos.route_simulations[i].estimated_amount_out.parse().unwrap()
                };
                let result = construct_raydium_instructions(swap_params);
                for instruction in result {
                    swap_instructions.push(instruction);
                }
            }
            DexLabel::RAYDIUM_CLMM => {
                info!("⚠️ RAYDIUM_CLMM TX NOT IMPLEMENTED");
            }
            DexLabel::ORCA_WHIRLPOOLS => {

            }
            DexLabel::ORCA => {
                info!("⚠️ ORCA TX NOT IMPLEMENTED");
            }
        }
    }
    return swap_instructions;
}
