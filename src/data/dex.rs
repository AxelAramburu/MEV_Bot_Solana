use core::panic;
use std::{collections::HashMap, str::FromStr, sync::Arc};

use ethers::abi::{parse_abi, Contract, Abi};
use ethers::prelude::BaseContract;
use ethers::providers::{call_raw::RawCall, Provider, Ws};
use ethers::types::{spoof, BlockNumber, TransactionRequest, H160, U256, U64};
use ethers::contract::abigen;
use serde_json;

use ndarray::{Array, Array3, Array4, Dim, Ix4};


use crate::common::pools::{Pool, DexVariant};
use crate::common::tokens::{get_token_info, Token, TokenInfo};
use crate::common::utils::{create_new_wallet, double_u128_in_u256, to_h160, u256_in_double_u128};

pub struct PoolInfoUniswapV2 {
    reserve0: U256,
    reserve1: U256,
    blockTimestampLast: U256
}
pub struct PoolInfoUniswapV3 {
    sqrtPriceX96: U256,
    tick: i32,
    observationIndex: u16,
    observationCardinality: u16,
    observationCardinalityNext: u16,
    feeProtocol: u8,
    unlocked: bool
}
#[derive(Clone)]
pub struct Dex {
    pub provider: Arc<Provider<Ws>>,
    pub block_number: U64,
    pub exchanges_list: Vec<String>,
    pub trading_symbols: HashMap<H160, TokenInfo>,
    pub tokens_map: HashMap<H160, Token>,
    pub pools: HashMap<H160, Pool>,
    pub storage_array: Array<u128, Dim<[usize; 4]>>,
    pub swap_paths: HashMap<String, String>,
 }

impl Dex {
    pub fn new(provider: Arc<Provider<Ws>>, trading_symbols: HashMap<H160, TokenInfo>, tokens_map: HashMap<H160, Token>, pools: HashMap<H160, Pool>, block_number: U64) -> Dex {
        let exchanges_list: Vec<String> = vec![
            "".to_string(), 
            "".to_string(), 
            "UniswapV2".to_string(), 
            "UniswapV3".to_string()
            ];
    
        let mut storage_array = Array4::<u128>::zeros((
            exchanges_list.len(),
            tokens_map.len(),
            tokens_map.len(),
            9                   
            // # decimals0, decimals1, reserve0, reserve1, sqrtPriceX96(lower 32 bytes), sqrtPriceX96 (higher 32 bytes), fee, token0_is_input, pool_index
            // 2 u128 values for potential overflow of sqrtPriceX96
        ));
    
        let mut swap_paths: HashMap<String, String> = HashMap::new();

        let provider = provider.clone();

        Dex {
            provider,
            block_number,
            exchanges_list,
            trading_symbols,
            tokens_map,
            pools,
            storage_array,
            swap_paths,
        }
    }
    
    pub async fn load(&mut self) {
        Self::_load_pool_data(self).await;
        Self::_generate_swap_paths(&self);
    }
    
    pub async fn _load_pool_data(&mut self) {
        for (pool_idx, pool) in self.pools.iter().enumerate() {
            
            let token0: &Token = self.tokens_map
            .iter()
            .find(|(_key, value)| pool.1.token0 == value.address)
            .map(|(key, _value)| _value)
            .unwrap();

            let token1: &Token = self.tokens_map
            .iter()
            .find(|(_key, value)| pool.1.token1 == value.address)
            .map(|(key, _value)| _value)
            .unwrap();

            let provider = Arc::clone(&self.provider);  
            
            if pool.1.dex == DexVariant::UniswapV2 {

                let pool_infos_result = tokio::task::spawn(Self::get_pool_info_Uniswap_v2(
                    provider,
                    self.block_number.clone(),
                    pool.1.address,
                )).await;

                let pool_infos = match pool_infos_result {
                    Ok(result) => result,
                    Err(err) => {
                        // Handle the error here, for example, printing an error message and returning early
                        eprintln!("Error getting pool info: {}", err);
                        return; // or any other action appropriate for your application flow
                    }
                };
                let pools_infos_unwrap = match pool_infos {
                    Ok(result) => result,
                    Err(err) => {
                        // Handle the error here, for example, printing an error message and returning early
                        eprintln!("Error getting pool info: {}", err);
                        return; // or any other action appropriate for your application flow
                    }
                };

                
                let decimals0 = token0.decimals;
                let decimals1 = token1.decimals;
                
                let reserve0 = pools_infos_unwrap.reserve0;
                let reserve1 = pools_infos_unwrap.reserve1;
                
                let fee = pool.1.fee;

                // On data_idx,
                // data[6] = 1 = token0 In
                // data[6] = 0 = token1 In
                let data_idx1 = [decimals0.into(), decimals1.into(), reserve0.as_u128(), reserve1.as_u128(), 0, 0, fee.into(), 1, pool_idx.try_into().unwrap()];
                let data_idx2 = [decimals0.into(), decimals1.into(), reserve0.as_u128(), reserve1.as_u128(), 0, 0, fee.into(), 0, pool_idx.try_into().unwrap()];

                for i in 0..8 {
                    // self.storage_array[[2, 2, 0, i]] = data_idx1[i];
                    self.storage_array[[2 as usize, token0.id as usize, token1.id as usize, i]] = data_idx1[i];
                }
                for i in 0..8 {
                    // self.storage_array[[2, 2, 0, i]] = data_idx1[i];
                    self.storage_array[[2 as usize, token1.id as usize, token0.id as usize, i]] = data_idx2[i];
                }
            } else if pool.1.dex == DexVariant::UniswapV3 {
                let pool_infos_result = tokio::task::spawn(Self::get_pool_info_Uniswap_v3(
                    provider,
                    self.block_number.clone(),
                    pool.1.address,
                )).await;

                let pool_infos = match pool_infos_result {
                    Ok(result) => result,
                    Err(err) => {
                        // Handle the error here, for example, printing an error message and returning early
                        eprintln!("Error getting pool info: {}", err);
                        return; // or any other action appropriate for your application flow
                    }
                };
                let pools_infos_unwrap = match pool_infos {
                    Ok(result) => result,
                    Err(err) => {
                        // Handle the error here, for example, printing an error message and returning early
                        eprintln!("Error getting pool info: {}", err);
                        return; // or any other action appropriate for your application flow
                    }
                };

                let decimals0 = token0.decimals;
                let decimals1 = token1.decimals;
                
                let sqrt_price_x96 = pools_infos_unwrap.sqrtPriceX96;

                let (u1, u2) = u256_in_double_u128(sqrt_price_x96);
                
                let fee = pool.1.fee;

                // On data_idx,
                // data[6] = 1 = token0 In
                // data[6] = 0 = token1 In
                let data_idx1 = [decimals0.into(), decimals1.into(), 0, 0, u1, u2, fee.into(), 1, pool_idx.try_into().unwrap()];
                let data_idx2 = [decimals0.into(), decimals1.into(), 0, 0, u1, u2, fee.into(), 0, pool_idx.try_into().unwrap()];

                for i in 0..8 {
                    // self.storage_array[[2, 2, 0, i]] = data_idx1[i];
                    self.storage_array[[3 as usize, token0.id as usize, token1.id as usize, i]] = data_idx1[i];
                }
                for i in 0..8 {
                    // self.storage_array[[2, 2, 0, i]] = data_idx1[i];
                    self.storage_array[[3 as usize, token1.id as usize, token0.id as usize, i]] = data_idx2[i];
                }

            }

        }
        
    }

    pub async fn get_pool_info_Uniswap_v2(
        provider: Arc<Provider<Ws>>,
        block_number: U64,
        pool_address: H160,
    ) -> Result<PoolInfoUniswapV2, anyhow::Error> {
        let owner = create_new_wallet().1;
    
        let mut state = spoof::state();
        state.account(owner).balance(U256::MAX).nonce(0.into());
    
        let request_address = create_new_wallet().1;
        state
            .account(request_address);
            // .code((*REQUEST_BYTECODE).clone());

        let contract = BaseContract::from(
            parse_abi(&["function getReserves() external view returns (uint112,uint112,uint32)"])
                .unwrap(),
        );

        let calldata = contract.encode("getReserves", ())?;
    
        let gas_price = U256::from(1000)
            .checked_mul(U256::from(10).pow(U256::from(9)))
            .unwrap();

        let tx = TransactionRequest::default()
            .from(owner)
            .to(pool_address)
            .value(U256::zero())
            .data(calldata.0)
            .nonce(U256::zero())
            .gas(5000000)
            .gas_price(gas_price)
            .into();

        let result = provider
            .call_raw(&tx)
            .state(&state)
            .block(block_number.into())
            .await?;

        let out: (U256, U256, U256) = contract.decode_output("getReserves", result)?;
        let pool_infos = PoolInfoUniswapV2 {
            reserve0: out.0,
            reserve1: out.1,
            blockTimestampLast: out.2,
        };
        Ok(pool_infos)
    }
    
    pub async fn get_pool_info_Uniswap_v3(
        provider: Arc<Provider<Ws>>,
        block_number: U64,
        pool_address: H160,
    ) -> Result<PoolInfoUniswapV3, anyhow::Error> {
        let owner = create_new_wallet().1;
    
        let mut state = spoof::state();
        state.account(owner).balance(U256::MAX).nonce(0.into());
    
        let request_address = create_new_wallet().1;
        state
            .account(request_address);
            // .code((*REQUEST_BYTECODE).clone());

        let contract = BaseContract::from(
            parse_abi(&["function slot0() external view returns (uint160,int24,uint16,uint16,uint16,uint8,bool)"])
                .unwrap(),
        );

        let calldata = contract.encode("slot0", ())?;
    
        let gas_price = U256::from(1000)
            .checked_mul(U256::from(10).pow(U256::from(9)))
            .unwrap();

        let tx = TransactionRequest::default()
            .from(owner)
            .to(pool_address)
            .value(U256::zero())
            .data(calldata.0)
            .nonce(U256::zero())
            .gas(5000000)
            .gas_price(gas_price)
            .into();

        let result = provider
            .call_raw(&tx)
            .state(&state)
            .block(block_number.into())
            .await?;

        let out: (U256, i32, u16, u16, u16, u8, bool) = contract.decode_output("slot0", result)?;
        let pool_infos = PoolInfoUniswapV3 {
                sqrtPriceX96: out.0,
                tick: out.1,
                observationIndex: out.2,
                observationCardinality: out.3,
                observationCardinalityNext: out.4,
                feeProtocol: out.5,
                unlocked: out.6
        };
        Ok(pool_infos)
    }
    
    pub fn _generate_swap_paths(&self) {
        let pools = self.pools.clone();
        let tokens_map = self.tokens_map.clone();
        Self::generate_swaps_with_pools(pools, tokens_map);
    }
    pub fn generate_swaps_with_pools(pools: HashMap<H160, Pool>, tokens: HashMap<H160, Token>) {

        //   {
        //     'source': 'dex',
        //     'type': 'event',
        //     'block': 17726053,
        //     'path': [[[0, 1, 5, 2, 1], [0, 0, 0, 0, 0]], [[0, 1, 5, 2, 0], [0, 0, 0, 0, 0]], [[0, 0, 5, 2, 1], [0, 0, 0, 0, 0]], [[0, 0, 5, 2, 0], [0, 0, 0, 0, 0]], [[0, 1, 5, 4, 1], [0, 1, 4, 2, 0]], [[0, 1, 5, 4, 1], [0, 0, 4, 2, 1]], [[0, 1, 5, 4, 1], [0, 0, 4, 2, 0]], [[0, 0, 5, 4, 1], [0, 1, 4, 2, 0]], [[0, 0, 5, 4, 1], [0, 0, 4, 2, 1]], [[0, 0, 5, 4, 1], [0, 0, 4, 2, 0]]],
        //     'pool_indexes': [[0], [5], [9], [15], [1, 6], [1, 10], [1, 16], [12, 6], [12, 10], [12, 16]],
        //     'symbol': 'ETH/USDT',
        //     'tag': ['ethereum-0', 'ethereum-1', 'ethereum-2', 'ethereum-3', 'ethereum-4', 'ethereum-5', 'ethereum-6', 'ethereum-7', 'ethereum-8', 'ethereum-9'],
        //     'price': [1908.0519148221722, 1910.8749837806993, 1911.117066215452, 1911.0445021506505, 1908.5838300538105, 1910.425470198242, 1911.5290103508617, 1908.4971959551506, 1910.3387525041794, 1911.4422425651003],
        //     'fee': [0.0004999999999999449, 0.0030000000000000027, 0.0004999999999999449, 0.0030000000000000027, 0.0030997000000000385, 0.0005999499999999047, 0.0030997000000000385, 0.0030997000000000385, 0.0005999499999999047, 0.0030997000000000385]
        //   }
        let mut swaps_dict: Vec<[i64; 4]> = Vec::new();
        
        //Generate swaps_dict, it's all swap possible
        for (pool_idx, pool) in pools.iter().enumerate() {
            let dex_id = pool.1.version.num() as i64;
            let pool_id = pool.1.id;
            let mut token0_id = 10000;
            let mut token1_id = 10000;
            for (token_idx, token) in tokens.iter().enumerate() {
                if token.1.address == pool.1.token0 {
                    token0_id = token.1.id;
                }
                if token.1.address == pool.1.token1 {
                    token1_id = token.1.id;
                }
                // if (token0_id == 10000 || token1_id == 10000) {
                //     panic!("Problem in swaps creation");
                // }
            }
            swaps_dict.push([dex_id, pool_id, token0_id, token1_id]);
            swaps_dict.push([dex_id, pool_id, token1_id, token0_id]);
        }


        //Generate possibles paths

        let mut swap_paths = Vec::new();
        let mut pool_indexes: Vec<Vec<i64>> = Vec::new();
        let mut swap_tags: Vec<String> = Vec::new();
        for (swap_idx_x, path_x) in swaps_dict.iter().enumerate() {
            for (swap_idx_y, path_y) in swaps_dict.iter().enumerate() {
                //If pool id are not the same and tokenOut_swap1 == tokenIn_swap2, add path
                if (path_x[1] != path_y[1]) && (path_x[3] == path_y[2]) {
                    let path = [path_x,path_y];
                    swap_paths.push(path);
                    let pool_index = vec![path_x[1], path_y[1]];
                    pool_indexes.push(pool_index);
                    let mut tag = String::from("path_");
                    let length = swap_paths.len().to_string();
                    let tag_swap: String = tag + &length;
                    swap_tags.push(tag_swap);
                }
            }
            let single_path = [path_x, &[0 as i64,0 as i64,0 as i64,0 as i64]];
            swap_paths.push(single_path);
            let single_pool_index = vec![path_x[1]];
            pool_indexes.push(single_pool_index);
            let mut tag = String::from("path_");
            let length = swap_paths.len().to_string();
            let tag_swap: String = tag + &length;
            swap_tags.push(tag_swap);
        }   

        println!("swap_dict {:?}", swaps_dict);
        println!("Possibles swap paths: {:?}", swap_paths.len());
        println!("swap_paths {:?}", swap_paths);
        println!("pool_indexes {:?}", pool_indexes);
        println!("swap_tags {:?}", swap_tags);
    }

    pub fn get_index(
        exchange: String,
        token0: String,
        token1: String,
    ) {

    }

}