// use std::rc::Rc;

// use anchor_client::Client;
// use anchor_client::{
//     solana_client::rpc_config::RpcSendTransactionConfig,
//     solana_sdk::{
//         commitment_config::CommitmentConfig,
//         signer::{keypair::*, Signer},
//     },
// };
// use anyhow::*;
// use clap::*;

// use lb_clmm::state::preset_parameters::PresetParameter;
// use anchor_client::solana_sdk::pubkey::Pubkey;
// use clap::*;
// use solana_sdk::signature::{read_keypair_file, Keypair};

// use crate::transactions::meteoradlmm_swap::{swap, SwapParameters};

// #[tokio::main]
// async fn main() -> Result<()> {
//     let cli = Cli::parse();

//     let payer =
//         read_keypair_file(cli.config_override.wallet).expect("Wallet keypair file not found");

//     println!("Wallet {:#?}", payer.pubkey());

//     let commitment_config = CommitmentConfig::confirmed();
//     let client = Client::new_with_options(
//         cli.config_override.cluster,
//         Rc::new(Keypair::from_bytes(&payer.to_bytes())?),
//         commitment_config,
//     );

//     let amm_program = client.program(lb_clmm::ID).unwrap();

//     let transaction_config: RpcSendTransactionConfig = RpcSendTransactionConfig {
//         skip_preflight: false,
//         preflight_commitment: Some(commitment_config.commitment),
//         encoding: None,
//         max_retries: None,
//         min_context_slot: None,
//     };

//     match cli.command {
//         Swap {
//             lb_pair,
//             amount_in,
//             swap_for_y,
//         } => {
//             let params = SwapParameters {
//                 amount_in,
//                 lb_pair,
//                 swap_for_y,
//             };
//             swap(params, &amm_program, transaction_config).await?;
//         }
//     };

//     Ok(())
// }

// /// Trade token X -> Y, or vice versa.
// pub struct Swap {
//     /// Address of the liquidity pair.
//     lb_pair: Pubkey,
//     /// Amount of token to be sell.
//     amount_in: u64,
//     /// Buy direction. true = buy token Y, false = buy token X.
//     #[clap(long)]
//     swap_for_y: bool,
// }