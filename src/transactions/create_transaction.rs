use anchor_spl::token::spl_token::error;
use itertools::Itertools;
use log::info;
use serde::{Deserialize, Serialize};
use solana_client::{rpc_client::RpcClient, rpc_config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig}};
use solana_sdk::{
    address_lookup_table::{instruction::{create_lookup_table, extend_lookup_table},
    state::AddressLookupTable, AddressLookupTableAccount}, commitment_config::CommitmentConfig, compute_budget::ComputeBudgetInstruction, instruction::Instruction, message::{v0, VersionedMessage}, pubkey::Pubkey, signature::{read_keypair_file, Keypair, Signer}, sysvar::instructions, transaction::VersionedTransaction
};
use anchor_spl::token::spl_token;
use solana_sdk::{transaction::Transaction};
use anyhow::Result;
use log::error;
use spl_associated_token_account::{get_associated_token_address};
use spl_associated_token_account::instruction::create_associated_token_account;
use std::{fs::{File, OpenOptions}, io::{BufReader, Read}, path::Path, thread::sleep, time::{Duration, Instant}};
use std::io::{BufWriter, Write};

use crate::{arbitrage::types::SwapPathResult, common::{constants::Env, utils::from_str}, markets::types::DexLabel, transactions::utils::check_tx_status};
use super::{meteoradlmm_swap::{construct_meteora_instructions, SwapParametersMeteora}, orca_whirpools_swap::{construct_orca_whirpools_instructions, SwapParametersOrcaWhirpools}, raydium_swap::{construct_raydium_instructions, SwapParametersRaydium}};

pub async fn create_swap_transaction(chain: ChainType, transaction_type: TransactionType, transaction_infos: SwapPathResult, lut_address: Pubkey) -> Result<()> {
    info!("🔄 Create transaction.... ");
    
    let env = Env::new();
    let rpc_url = if chain.clone() == ChainType::Mainnet { env.rpc_url } else { env.devnet_rpc_url };
    let rpc_client: RpcClient = RpcClient::new(rpc_url);

    let payer: Keypair = read_keypair_file(env.payer_keypair_path).expect("Wallet keypair file not found");
    info!("💳 Wallet {:#?}", payer.pubkey());

    match transaction_type {
        TransactionType::CreateLUT => {
            info!("🆔 Create/Send LUT transaction....");
            // Create LUT instruction
            let (create_lut_instruction, address_lut) = create_lut(chain.clone()).await;

            let txn_lut = Transaction::new_signed_with_payer(
                &vec![create_lut_instruction.clone()],
                Some(&payer.pubkey()),
                &vec![&payer],
                rpc_client.get_latest_blockhash().expect("Error in get latest blockhash"),
            );
        
            // let result_lut_creation = rpc_client.simulate_transaction(&txn_lut).unwrap().value;
            // info!("🧾 Simulate Tx LUT Creation & Extends Logs: {:#?}", result_lut_creation.logs);
            info!("🧾 Address LUT {:#?}", address_lut);

            let commitment_config = CommitmentConfig::confirmed();

            let transaction_config: RpcSendTransactionConfig = RpcSendTransactionConfig {
                skip_preflight: false,
                .. RpcSendTransactionConfig::default()
            };

            let signature = rpc_client.send_transaction_with_config(
                &txn_lut,
                transaction_config
            ).unwrap();

            if chain == ChainType::Devnet {
                info!("https://explorer.solana.com/tx/{}?cluster=devnet", signature);
            } else {
                info!("https://explorer.solana.com/tx/{}", signature);
            }

            let start = Instant::now();
            let mut counter = 0;
            loop {
                let confirmed = rpc_client.confirm_transaction(&signature)?;

                let status = rpc_client.get_signature_status(&signature)?;
                let sixty_secs = Duration::from_secs(60);
                if confirmed {
                    info!("✅ Transaction Confirmed with Confirmation");
                    break;
                }
                if status.is_some() {
                    info!("✅ Transaction Confirmed with Status");
                    break;
                }
                if start.elapsed() >= sixty_secs {
                    error!("❌ Transaction not confirmed");
                    break;
                }
                let ten_secs = Duration::from_secs(10);
                info!("⏳ {} seconds...", 10 * counter);
                sleep(ten_secs);
                counter += 1;
            }
        }
        TransactionType::CreateSwap => {
            info!("🆔 Create/Send Swap instruction....");
            // Construct Swap instructions
            let swap_instructions: Vec<InstructionDetails> = construct_transaction(transaction_infos).await;
            
            if swap_instructions.len() == 0 {
                error!("Error in create_transaction()");
                return Ok(());
            }
            
            // Create the extend LUT instructions
            let mut extend_instructions_vec: Vec<Instruction> = Vec::new();
            
            let mut instructions: Vec<Instruction> = swap_instructions.clone().into_iter().map(|instruc_details| instruc_details.instruction).collect();
            let si_details: Vec<String> = swap_instructions.clone().into_iter().map(|instruc_details| instruc_details.details).collect();
            info!("📋 Swap instructions: {:#?}", si_details);

            // let txn = Transaction::new_signed_with_payer(
            //     &instructions,
            //     Some(&payer.pubkey()),
            //     &vec![&payer],
            //     rpc_client.get_latest_blockhash().expect("Error in get latest blockhash"),
            // );

            let raw_lut_account = rpc_client.get_account(&lut_address)?;
            let address_lookup_table = AddressLookupTable::deserialize(&raw_lut_account.data)?;
            let address_lookup_table_account = AddressLookupTableAccount {
                key: lut_address,
                addresses: address_lookup_table.addresses.to_vec(),
            };

            println!("Address in lookup_table: {:#?}", address_lookup_table_account);
            println!("Address in lookup_table: {}", address_lookup_table_account.addresses.len());

            let all_instructions: Vec<Instruction> = vec![extend_instructions_vec, instructions.clone()].concat();

            println!("instructions: {:?}", instructions);

            let tx = VersionedTransaction::try_new(
                VersionedMessage::V0(v0::Message::try_compile(
                    &payer.pubkey(),
                    &instructions,
                    &[address_lookup_table_account],
                    rpc_client.get_latest_blockhash()?,
                )?),
                &[&payer],
            )?;
            // println!("TX: {:?}", tx);

            // Simulate transaction
            let config = RpcSimulateTransactionConfig {
                sig_verify: false,
                .. RpcSimulateTransactionConfig::default()
            };
            
            let result = rpc_client.simulate_transaction_with_config(&tx, config).unwrap().value;
            info!("🧾 Simulate Tx Swap Logs: {:#?}", result.logs);

            // let transaction_config: RpcSendTransactionConfig = RpcSendTransactionConfig {
            //     skip_preflight: false,
            //     .. RpcSendTransactionConfig::default()
            // };

            // let signature = rpc_client.send_transaction_with_config(
            //     &tx,
            //     transaction_config
            // ).unwrap();

            // if chain == ChainType::Devnet {
            //     info!("https://explorer.solana.com/tx/{}?cluster=devnet", signature);
            // } else {
            //     info!("https://explorer.solana.com/tx/{}", signature);
            // }

            // check_tx_status(chain, signature);
        }
    }

    // println!("lut_address: {}", lut_address);
    // println!("Txn: {:?}", txn);

    Ok(())
}

pub async fn create_ata_extendlut_transaction(chain: ChainType, transaction_infos: SwapPathResult, lut_address: Pubkey, tokens: Vec<Pubkey>) -> Result<()> {
    info!("🔄 Create ATA/Extend LUT transaction.... ");
    
    let env = Env::new();
    let rpc_url = if chain.clone() == ChainType::Mainnet { env.rpc_url } else { env.devnet_rpc_url };
    let rpc_client: RpcClient = RpcClient::new(rpc_url);

    let payer: Keypair = read_keypair_file(env.payer_keypair_path).expect("Wallet keypair file not found");
    info!("💳 Wallet {:#?}", payer.pubkey());

    let mut vec_pda_instructions: Vec<Instruction> = Vec::new();

    //Create Pda/Ata accounts
    for token in tokens {
        let pda_user_token = get_associated_token_address(
            &payer.pubkey(),
            &token,
        );
        match rpc_client.get_account(&pda_user_token) {
            Ok(account) => {
                info!("🟢 PDA for {} already exist !", token);
            }
            Err(error) => {
                info!("👷‍♂️ PDA creation...");
                let create_pda_instruction = create_associated_token_account(
                    &payer.pubkey(),
                    &payer.pubkey(),
                    &token,
                    &spl_token::id()
                );
                vec_pda_instructions.push(create_pda_instruction);
            }
        }
    }


    // Create the extend LUT instructions

    //Check if lookup table is already exist
    info!("👷‍♂️👷‍♂️👷‍♂️ Implement Logic here");

    // Construct Swap instructions
    let swap_instructions: Vec<InstructionDetails> = construct_transaction(transaction_infos).await;

        //👷‍♂️👷‍♂️👷‍♂️ If market LUT already exist, delete instruction
        //👷‍♂️👷‍♂️👷‍♂️ And if it give us an empy instruction, return address of LUT with no need to create new

    // info!("📋 Swap instructions: {:#?}", si_details);

    let mut vec_details_extend_instructions: Vec<InstructionDetails> = Vec::new();

    for (i, instr) in swap_instructions.clone().iter().enumerate() {
        if instr.instruction.accounts.len() == 0 {
            continue;
        }
        if instr.clone().market.is_none() {
            error!("Instruction market is None");
            return Ok(());
        }
        let accounts: Vec<Pubkey> = instr.instruction.accounts.iter().map(|account| account.pubkey).collect();
        // println!("Accounts: {:?}", accounts);

        let extend_instruction = extend_lookup_table(
            lut_address, 
            payer.pubkey(), 
            Some(payer.pubkey()), 
            accounts
        );
        vec_details_extend_instructions.push(InstructionDetails{instruction: extend_instruction, details: "Extend Instruction".to_string(), market: instr.clone().market});
    }

    let vec_extend_instructions: Vec<Instruction> = vec_details_extend_instructions.clone().into_iter().map(|instruc_details| instruc_details.instruction).collect();

    let vec_all_instructions: Vec<Instruction> = vec![vec_pda_instructions, vec_extend_instructions].concat();

    let txn = Transaction::new_signed_with_payer(
        &vec_all_instructions,
        Some(&payer.pubkey()),
        &vec![&payer],
        rpc_client.get_latest_blockhash().expect("Error in get latest blockhash"),
    );

    let transaction_config: RpcSendTransactionConfig = RpcSendTransactionConfig {
        skip_preflight: false,
        .. RpcSendTransactionConfig::default()
    };

    let signature = rpc_client.send_transaction_with_config(
        &txn,
        transaction_config
    ).unwrap();

    if chain == ChainType::Devnet {
        info!("https://explorer.solana.com/tx/{}?cluster=devnet", signature);
    } else {
        info!("https://explorer.solana.com/tx/{}", signature);
    }
    
    let tx_confirmed = check_tx_status(chain, signature).await?;

    if tx_confirmed {
        for details_instruction in vec_details_extend_instructions {
            write_lut_for_market(details_instruction.market.unwrap().address, lut_address, false);
        }
    }
    
    Ok(())
}
pub async fn construct_transaction(transaction_infos: SwapPathResult) -> Vec<InstructionDetails> {
    
    let mut swap_instructions: Vec<InstructionDetails> = Vec::new();
    let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(1_400_000);
    swap_instructions.push(InstructionDetails{ instruction: compute_budget_ix, details: "Compute Budget Instruction".to_string(), market: None });
    
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
                if result.len() == 0 {
                    let empty_array: Vec<InstructionDetails> = Vec::new();
                    error!("Error in Meteora Instruction");
                    return empty_array;
                }
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
                if result.len() == 0 {
                    let empty_array: Vec<InstructionDetails> = Vec::new();
                    error!("Error in Raydium Instruction");
                    return empty_array;
                }
                for instruction in result {
                    swap_instructions.push(instruction);
                }
            }
            DexLabel::RAYDIUM_CLMM => {
                info!("⚠️ RAYDIUM_CLMM TX NOT IMPLEMENTED");
            }
            DexLabel::ORCA_WHIRLPOOLS => {
                let swap_params: SwapParametersOrcaWhirpools = SwapParametersOrcaWhirpools{
                    whirpools: from_str(transaction_infos.route_simulations[i].pool_address.as_str()).unwrap(),
                    input_token: from_str(route_sim.token_in.as_str()).unwrap(),
                    output_token: from_str(route_sim.token_out.as_str()).unwrap(),
                    amount_in: transaction_infos.route_simulations[i].amount_in,
                    minimum_amount_out: transaction_infos.route_simulations[i].estimated_amount_out.parse().unwrap()
                };
                let result = construct_orca_whirpools_instructions(swap_params).await;
                // Return len 0 to handle error case in swap
                if result.len() == 0 {
                    let empty_array: Vec<InstructionDetails> = Vec::new();
                    error!("Error in Orca_Whirpools Instruction");
                    return empty_array;
                }
                for instruction in result {
                    swap_instructions.push(instruction);
                }
            }
            DexLabel::ORCA => {
                info!("⚠️ ORCA TX NOT IMPLEMENTED");
            }
        }
    }
    return swap_instructions;
}

pub async fn create_lut(chain: ChainType) -> (Instruction, Pubkey) {

    let env = Env::new();
    let rpc_url = if chain == ChainType::Mainnet { env.rpc_url } else { env.devnet_rpc_url };
    let rpc_client: RpcClient = RpcClient::new(rpc_url);
    let payer: Keypair = read_keypair_file(env.payer_keypair_path).expect("Wallet keypair file not found");

    //Create Address Lookup Table (LUT acronym)
    let slot = rpc_client.get_slot_with_commitment(CommitmentConfig::finalized()).expect("Error in get slot");
    let (create_lut_instruction, lut_address) = create_lookup_table(
        payer.pubkey(),
        payer.pubkey(),
        slot - 200
    );

    return (create_lut_instruction, lut_address);
}

pub async fn is_available_lut(chain: ChainType, lut_address: Pubkey) -> Result<()> {
    info!("🚚 Check if LUT address is available to craft a transaction...");
    let env = Env::new();
    let rpc_url = if chain == ChainType::Mainnet { env.rpc_url } else { env.devnet_rpc_url };
    let rpc_client: RpcClient = RpcClient::new(rpc_url);
    let payer: Keypair = read_keypair_file(env.payer_keypair_path).expect("Wallet keypair file not found");

    let raw_lut_account = rpc_client.get_account(&lut_address)?;
    let address_lookup_table = AddressLookupTable::deserialize(&raw_lut_account.data)?;
    
    let lut_length = address_lookup_table.addresses.len();

    Ok(())
}

pub fn write_lut_for_market(market:Pubkey, lut_address: Pubkey, is_test: bool) -> Result<()> {
    let mut path = "";
    if is_test {
        path = "src/transactions/cache/lut_addresses_test.json";
    } else {
        path = "src/transactions/cache/lut_addresses.json";
    }
    let file_exist = Path::new(path).is_file();
    if !file_exist {
        File::create(path)?;

        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let mut writer = BufWriter::new(&file);

        let mut lut_file = VecLUTFile{value: Vec::new()};
        let field = LUTFile{market: market.to_string(), lut_address: lut_address.to_string()};
        lut_file.value.push(field);
        println!("LUT File: {:?}", lut_file);
        writer.write_all(serde_json::to_string(&lut_file)?.as_bytes())?;
        writer.flush()?;
        println!("Data written to 'lut_addresses.json' successfully.");
    } else {
        let file_read = OpenOptions::new().read(true).write(true).open(path)?;
        let mut lut_file: VecLUTFile = serde_json::from_reader(&file_read).unwrap();
        
        let file_write = OpenOptions::new().read(true).write(true).truncate(true).open(path)?;
        let mut writer = BufWriter::new(&file_write);
        // let mut lut_file = VecLUTFile{value: Vec::new()};
        let field = LUTFile{market: market.to_string(), lut_address: lut_address.to_string()};
        lut_file.value.push(field);
        println!("LUT File: {:?}", lut_file);
        writer.write_all(serde_json::to_string(&lut_file)?.as_bytes())?;
        writer.flush()?;
        println!("Data written to 'lut_addresses.json' successfully.");
    }


    Ok(())
}

////////////////////

#[derive(Debug, Clone)]
pub struct InstructionDetails {
    pub instruction: Instruction,
    pub details: String,
    pub market: Option<MarketInfos>
}
#[derive(Debug, Clone)]
pub struct MarketInfos {
    pub dex_label: DexLabel,
    pub address: Pubkey,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VecLUTFile {
    pub value: Vec<LUTFile>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LUTFile {
    pub market: String,
    pub lut_address: String,
}

pub enum TransactionType {
    CreateLUT,
    CreateSwap,
}

#[derive(PartialEq, Clone)]
pub enum ChainType {
    Mainnet,
    Devnet,
}
