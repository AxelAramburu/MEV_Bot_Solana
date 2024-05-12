use num::bigint::ToBigUint;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;
use rust_decimal_macros::dec;
use num::BigUint;

pub fn from_x64_orca_wp(num: u128, decimal_a: u8, decimal_b: u8) -> Decimal {
    println!("numX64: {:?}", num);
    // let num_str = num.to_biguint().expect("Potential overflow on from_x64_orca_wp");

    let num_dec = Decimal::from_u128(num).unwrap();
    let mul_x64 = 2f64.powf(-64.0);
    let from_x64 = num_dec.checked_mul(Decimal::from_f64_retain(mul_x64).unwrap()).unwrap();
    let price = from_x64.checked_powd(dec!(2)).unwrap().checked_mul(Decimal::from_f64_retain(10f64.powi((decimal_a - decimal_b) as i32)).unwrap());
    return price.unwrap();


    // public static fromX64(num: BN): Decimal {
    //     return new Decimal(num.toString()).mul(Decimal.pow(2, -64));
    //   }

    // public static sqrtPriceX64ToPrice(
    //     sqrtPriceX64: BN,
    //     decimalsA: number,
    //     decimalsB: number,
    //   ): Decimal {
    //     return MathUtil.fromX64(sqrtPriceX64)
    //       .pow(2)
    //       .mul(Decimal.pow(10, decimalsA - decimalsB));
    //   }
    
}