//Taken here: https://github.com/MeteoraAg/dlmm-sdk/blob/main/cli/src/instructions/swap.rs

use std::ops::Deref;
use std::rc::Rc;

use anchor_client::solana_client::rpc_config::RpcSendTransactionConfig;
use anchor_client::solana_sdk::compute_budget::ComputeBudgetInstruction;
use anchor_client::Client;
use anchor_client::{solana_sdk::pubkey::Pubkey, solana_sdk::signer::Signer, Program};
use anchor_lang::solana_program::instruction::AccountMeta;
use anchor_spl::associated_token::get_associated_token_address;

use borsh::BorshDeserialize;
// use anyhow::*;
use lb_clmm::accounts;
use lb_clmm::instruction;

use lb_clmm::utils::pda::{derive_bin_array_bitmap_extension, derive_bin_array_pda};
use log::info;
use raydium_amm::instruction::swap_base_in;
use solana_client::rpc_client::RpcClient;
use solana_sdk::account::ReadableAccount;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::Instruction;
use solana_sdk::signature::{read_keypair_file, Keypair};

use crate::common::constants::Env;
use crate::common::utils::from_str;
use crate::markets::raydium::AmmInfo;

use super::utils::get_keys_for_market;

#[derive(Debug, Clone)]
pub struct SwapParametersRaydium {
    pub pool: Pubkey,
    pub input_token_mint: Pubkey,
    pub output_token_mint: Pubkey,
    pub amount_in: u64,
    pub swap_for_y: bool,
}
// Function are imported from Raydium library, you can see here: 
// https://github.com/raydium-io/raydium-library
pub fn construct_raydium_instructions(params: SwapParametersRaydium, transaction_config: RpcSendTransactionConfig, min_amount_out: u64) -> (Instruction) {
    let SwapParametersRaydium {
        pool,
        input_token_mint,
        output_token_mint,
        amount_in,
        swap_for_y,
    } = params;

    let env = Env::new();
    let payer = read_keypair_file(env.payer_keypair_path).expect("Wallet keypair file not found");
    
    let commitment_config = CommitmentConfig::confirmed();
    let client = Client::new_with_options(
        anchor_client::Cluster::Mainnet,
        Rc::new(Keypair::from_bytes(&payer.to_bytes()).expect("Payer error in client creation")),
        commitment_config,
    );
    let amm_program = client.program(from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").expect("Error unwrap Raydium program")).unwrap();
    //Devnet : HWy1jotHpo6UqeQxx49dpYYdQB8wj9Qk9MdxwjLvDHB8
    
    let rpc_client: RpcClient = RpcClient::new(env.rpc_url);
    let pool_account = rpc_client.get_account(&pool).unwrap();
    let pool_state = AmmInfo::try_from_slice(&pool_account.data).unwrap();

    let authority = raydium_amm::processor::Processor::authority_id(
        &amm_program.id(),
        raydium_amm::processor::AUTHORITY_AMM,
        pool_state.nonce as u8,
    ).unwrap();

    // load market keys
    let market_keys = get_keys_for_market(
        &rpc_client,
        &pool_state.market_program,
        &pool_state.market,
    ).unwrap();

    let pda_user_source = get_associated_token_address(
        &payer.pubkey(),
        &input_token_mint,
    );
    let pda_user_destination = get_associated_token_address(
        &payer.pubkey(),
        &output_token_mint,
    );

    let swap_instruction = swap_base_in(
        &amm_program.id(),
        &pool,
        &authority,
        &pool_state.open_orders,
        &pool_state.coin_vault,
        &pool_state.pc_vault,
        &pool_state.market_program,
        &pool_state.market,
        &market_keys.bids,
        &market_keys.asks,
        &market_keys.event_q,
        &market_keys.coin_vault,
        &market_keys.pc_vault,
        &market_keys.vault_signer_key,
        &pda_user_source,
        &pda_user_destination,
        &payer.pubkey(),
        amount_in,
        min_amount_out,
    ).expect("Error in Raydium swap instruction construction");

    return (swap_instruction);
}
