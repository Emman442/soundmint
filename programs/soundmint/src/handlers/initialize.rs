use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(context: Context<Initialize>) -> Result<()> {
    let treasury = &mut context.accounts.treasury;
    let clock = Clock::get()?;
    
    treasury.authority = context.accounts.authority.key();
    treasury.treasury_wallet = context.accounts.treasury_wallet.key();
    treasury.mint_fee = 10_000_000; // 0.01 SOL default fee
    treasury.platform_fee_basis_points = 500; // 5% default platform fee
    treasury.total_revenue_collected = 0;
    treasury.created_at = clock.unix_timestamp;
    treasury.updated_at = clock.unix_timestamp;
    treasury.bump = context.bumps.treasury;
    
    msg!("SoundMint platform initialized");
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// CHECK: This is the wallet that will receive treasury funds
    pub treasury_wallet: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = authority,
        space = Treasury::INIT_SPACE,
        seeds = [b"treasury"],
        bump
    )]
    pub treasury: Account<'info, Treasury>,
    
    pub system_program: Program<'info, System>,
}