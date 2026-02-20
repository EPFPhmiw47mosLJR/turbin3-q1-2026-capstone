use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub first_price: u128,
    pub reserve_ratio: u128,
    pub base_fee_bps: u128,
    pub discount_bps: u128,
}
