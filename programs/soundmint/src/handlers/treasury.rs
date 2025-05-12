use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::CustomError;
use crate::constants::*;

pub fn update_treasury_config(
    context: Context<UpdateTreasuryConfigAccountConstraints>,
    mint_fee: Option<u64>,
    platform_fee_basis_points: Option<u16>,
    new_treasury_wallet: Option<Pubkey>
) -> Result<()> {
    let treasury = &mut context.accounts.treasury;
    let clock = Clock::get()?;
    
    // Only allow changes if authority is the signer
    require!(
        treasury.authority == context.accounts.authority.key(),
        CustomError::Unauthorized
    );
    
    if let Some(fee) = mint_fee {
        treasury.mint_fee = fee;
    }
    
    if let Some(fee_basis_points) = platform_fee_basis_points {
        require!(
            fee_basis_points <= TOTAL_BASIS_POINTS,
            CustomError::InvalidFeePercentage
        );
        treasury.platform_fee_basis_points = fee_basis_points;
    }
    
    if let Some(new_wallet) = new_treasury_wallet {
        treasury.treasury_wallet = new_wallet;
    }
    
    treasury.updated_at = clock.unix_timestamp;
    
    msg!("Treasury configuration updated");
    Ok(())
}

pub fn withdraw_treasury_funds(
    context: Context<WithdrawTreasuryFundsAccountConstraints>,
    amount: u64
) -> Result<()> {
    require!(amount > 0, CustomError::InvalidAmount);
    
    let treasury = &context.accounts.treasury;
    
    // Only allow withdrawals by treasury authority
    require!(
        treasury.authority == context.accounts.authority.key(),
        CustomError::Unauthorized
    );
    
    // Check that the wallet receiving funds is the treasury wallet
    require!(
        context.accounts.treasury_wallet.key() == treasury.treasury_wallet,
        CustomError::InvalidTreasuryWallet
    );
    
    // Check that there are sufficient funds in the program
    let program_balance = **context.accounts.program_account.lamports.borrow();
    require!(program_balance >= amount, CustomError::InsufficientFunds);
    
    // Transfer funds
    let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
        &context.accounts.program_account.key(),
        &context.accounts.treasury_wallet.key(),
        amount
    );
    
    anchor_lang::solana_program::program::invoke_signed(
        &transfer_instruction,
        &[
            context.accounts.program_account.to_account_info(),
            context.accounts.treasury_wallet.to_account_info(),
            context.accounts.system_program.to_account_info(),
        ],
        &[&[TREASURY_SEED, &[treasury.bump]]]
    )?;
    
    msg!("Withdrawn {} lamports from treasury", amount);
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateTreasuryConfigAccountConstraints<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury.bump,
        constraint = treasury.authority == authority.key() @ CustomError::Unauthorized
    )]
    pub treasury: Account<'info, Treasury>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawTreasuryFundsAccountConstraints<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        seeds = [TREASURY_SEED],
        bump = treasury.bump,
        constraint = treasury.authority == authority.key() @ CustomError::Unauthorized
    )]
    pub treasury: Account<'info, Treasury>,
    
    /// CHECK: Program's own account to withdraw from
    #[account(mut)]
    pub program_account: UncheckedAccount<'info>,
    
    /// CHECK: Treasury wallet to receive funds
    #[account(
        mut,
        constraint = treasury_wallet.key() == treasury.treasury_wallet @ CustomError::InvalidTreasuryWallet
    )]
    pub treasury_wallet: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}