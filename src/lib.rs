pub mod common;
pub mod strategies;
pub mod markets;
pub mod arbitrage;
pub mod transactions;

use std::env;

mod tests {
    use solana_sdk::pubkey::Pubkey;

    use crate::transactions::create_transaction::write_lut_for_market;

    #[test]
    fn write_in_write_lut_for_market() {
        let market: Pubkey = Pubkey::new_unique();
        let lut_address: Pubkey = Pubkey::new_unique();
        let _ = write_lut_for_market(market, lut_address, true);
        let market2: Pubkey = Pubkey::new_unique();
        let lut_address2: Pubkey = Pubkey::new_unique();
        let _ = write_lut_for_market(market2, lut_address2, true);
    }
}