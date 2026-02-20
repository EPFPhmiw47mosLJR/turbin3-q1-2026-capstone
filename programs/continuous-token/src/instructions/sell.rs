use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    burn, transfer_checked, Burn, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::{state::Config, ContinuousTokenError};

#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub config: Account<'info, Config>,

    #[account(mint::token_program = token_program_rt)]
    pub mint_rt: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [b"ct", config.seed.to_le_bytes().as_ref()],
        bump,
        mint::authority = config,
        mint::token_program = token_program_ct,
    )]
    pub mint_ct: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_rt,
        associated_token::authority = config,
        associated_token::token_program = token_program_rt,
    )]
    pub vault_rt: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_ct,
        associated_token::authority = config,
        associated_token::token_program = token_program_ct,
    )]
    pub vault_ct_locked: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_ct,
        associated_token::authority = seller,
        associated_token::token_program = token_program_ct,
    )]
    pub seller_ct_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_rt,
        associated_token::authority = seller,
        associated_token::token_program = token_program_rt,
    )]
    pub seller_rt_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program_rt: Interface<'info, TokenInterface>,
    pub token_program_ct: Interface<'info, TokenInterface>,
}

// CT in -> RT out
impl<'info> Sell<'info> {
    pub fn sell(&mut self, amount: u64) -> Result<()> {
        require!(
            self.seller_ct_ata.amount >= amount,
            ContinuousTokenError::InsufficientBalance
        );
        require!(amount > 0, ContinuousTokenError::InvalidAmount);

        let amount_u128 = amount as u128;

        let fee = amount_u128
            .checked_mul(self.config.base_fee_bps as u128)
            .ok_or(ContinuousTokenError::Overflow)?
            .checked_div(10_000)
            .ok_or(ContinuousTokenError::Underflow)?;
        let fee_u64: u64 = fee.try_into().map_err(|_| ContinuousTokenError::Overflow)?;

        let net_amount = amount_u128
            .checked_sub(fee)
            .ok_or(ContinuousTokenError::Underflow)?;
        let net_amount_u64 = net_amount
            .try_into()
            .map_err(|_| ContinuousTokenError::Overflow)?;

        let user_rt = Self::bonding_curve_sell(
            self.config.first_price,
            self.config.reserve_ratio_bps,
            self.mint_ct.supply,
            self.vault_rt.amount,
            amount_u128,
        )?;
        let user_rt_u64 = user_rt
            .try_into()
            .map_err(|_| ContinuousTokenError::Overflow)?;

        {
            // Transfer fee to locked
            let cpi_program = self.token_program_ct.to_account_info();

            let cpi_accounts = TransferChecked {
                from: self.seller_ct_ata.to_account_info(),
                mint: self.mint_ct.to_account_info(),
                to: self.vault_ct_locked.to_account_info(),
                authority: self.seller.to_account_info(),
            };

            let ctx = CpiContext::new(cpi_program, cpi_accounts);

            transfer_checked(ctx, fee_u64, self.mint_ct.decimals)?;
        }

        {
            // Burn
            let cpi_program = self.token_program_ct.to_account_info();

            let cpi_accounts = Burn {
                mint: self.mint_ct.to_account_info(),
                from: self.seller_ct_ata.to_account_info(),
                authority: self.seller.to_account_info(),
            };

            let ctx = CpiContext::new(cpi_program, cpi_accounts);

            burn(ctx, net_amount_u64)?;
        }

        {
            // Transfer reserve to user
            let seed_bytes = self.config.seed.to_le_bytes();
            let signer_seeds: &[&[&[u8]]] =
                &[&[b"config", seed_bytes.as_ref(), &[self.config.bump]]];

            let cpi_program = self.token_program_rt.to_account_info();

            let cpi_accounts = TransferChecked {
                from: self.vault_rt.to_account_info(),
                mint: self.mint_rt.to_account_info(),
                to: self.seller_rt_ata.to_account_info(),
                authority: self.config.to_account_info(),
            };

            let ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

            transfer_checked(ctx, user_rt_u64, self.mint_rt.decimals)?;
        }

        Ok(())
    }

    fn bonding_curve_sell(
        k: u128,
        reserve_ratio_bps: u16,
        supply: u64,
        reserve: u64,
        amount: u128,
    ) -> Result<u128> {
        todo!()
    }
}
