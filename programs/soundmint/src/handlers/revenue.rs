use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::CustomError;
use crate::constants::*;

pub fn track_revenue(
    context: Context<TrackRevenueAccountConstraints>,
    amount: u64,
    source: String,
    description: String
) -> Result<()> {
    require!(amount > 0, CustomError::InvalidAmount);
    require!(source.len() <= 20, CustomError::StringTooLong);
    require!(description.len() <= 100, CustomError::StringTooLong);
    
    let clock = Clock::get()?;
    let revenue_tracker = &mut context.accounts.revenue_tracker;
    let royalty_split = &mut context.accounts.royalty_split;
    
    // Create a new transaction record
    let transaction = RevenueTransaction {
        amount,
        source: source.clone(),
        description: description.clone(),
        timestamp: clock.unix_timestamp,
    };
    
    // Update the revenue tracker
    revenue_tracker.total_revenue = revenue_tracker.total_revenue.checked_add(amount).unwrap();
    
    // Update specific revenue category
    if source == "streaming" {
        revenue_tracker.streaming_revenue = revenue_tracker.streaming_revenue.checked_add(amount).unwrap();
    } else if source == "sales" {
        revenue_tracker.sales_revenue = revenue_tracker.sales_revenue.checked_add(amount).unwrap();
    } else {
        revenue_tracker.other_revenue = revenue_tracker.other_revenue.checked_add(amount).unwrap();
    }
    
    revenue_tracker.transactions.push(transaction);
    revenue_tracker.last_revenue_timestamp = clock.unix_timestamp;
    
    // Update royalty split total revenue
    royalty_split.total_revenue_collected = royalty_split.total_revenue_collected.checked_add(amount).unwrap();
    royalty_split.last_revenue_timestamp = clock.unix_timestamp;
    
    msg!("Revenue tracked: {} lamports from {}", amount, source);
    Ok(())
}

pub fn claim_revenue(
    context: Context<ClaimRevenueAccountConstraints>
) -> Result<()> {
    let clock = Clock::get()?;
    let royalty_nft = &mut context.accounts.royalty_nft;
    let royalty_split = &context.accounts.royalty_split;
    let treasury = &context.accounts.treasury;
    
    // Calculate claimable amount
    let total_revenue = royalty_split.total_revenue_collected;
    let share_percentage = royalty_nft.share_basis_points as u64;
    let platform_fee_percentage = treasury.platform_fee_basis_points as u64;
    
    // Calculate revenue share (total * share_percentage / TOTAL_BASIS_POINTS)
    let gross_share = total_revenue
        .checked_mul(share_percentage).unwrap()
        .checked_div(TOTAL_BASIS_POINTS as u64).unwrap();
    
    // Calculate amount already claimed
    let already_claimed = royalty_nft.amount_claimed;
    
    // Calculate net claimable amount
    let claimable_amount = gross_share.checked_sub(already_claimed).unwrap();
    require!(claimable_amount > 0, CustomError::NoRevenueToClaim);
    
    // Calculate platform fee
    let platform_fee = claimable_amount
        .checked_mul(platform_fee_percentage).unwrap()
        .checked_div(TOTAL_BASIS_POINTS as u64).unwrap();
    
    // Final amount after platform fee
    let final_amount = claimable_amount.checked_sub(platform_fee).unwrap();
    require!(final_amount > 0, CustomError::AmountTooSmall);
    
    // Transfer platform fee to treasury
    if platform_fee > 0 {
        let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
            &context.accounts.payer.key(),
            &treasury.treasury_wallet,
            platform_fee
        );
        
        anchor_lang::solana_program::program::invoke(
            &transfer_instruction,
            &[
                context.accounts.payer.to_account_info(),
                context.accounts.treasury_wallet.to_account_info(),
                context.accounts.system_program.to_account_info(),
            ]
        )?;
    }
    
    // Transfer funds to the royalty NFT holder
    let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
        &context.accounts.payer.key(),
        &context.accounts.authority.key(),
        final_amount
    );
    
    anchor_lang::solana_program::program::invoke(
        &transfer_instruction,
        &[
            context.accounts.payer.to_account_info(),
            context.accounts.authority.to_account_info(),
            context.accounts.system_program.to_account_info(),
        ]
    )?;
    
    // Update royalty NFT state
    royalty_nft.amount_claimed = royalty_nft.amount_claimed.checked_add(claimable_amount).unwrap();
    royalty_nft.last_claimed_at = clock.unix_timestamp;
    
    msg!("Revenue claimed: {} lamports", final_amount);
    Ok(())
}

#[derive(Accounts)]
pub struct TrackRevenueAccountConstraints<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        seeds = [MASTER_NFT_SEED, master_nft.mint.as_ref()],
        bump = master_nft.bump
    )]
    pub master_nft: Account<'info, MasterNft>,
    
    #[account(
        mut,
        seeds = [ROYALTY_SPLIT_SEED, master_nft.key().as_ref()],
        bump = royalty_split.bump,
        constraint = royalty_split.master_nft == master_nft.key() @ CustomError::InvalidRoyaltySplit
    )]
    pub royalty_split: Account<'info, RoyaltySplit>,
    
    #[account(
        init_if_needed,
        payer = authority,
        space = RevenueTracker::INIT_SPACE,
        seeds = [REVENUE_TRACKER_SEED, master_nft.key().as_ref()],
        bump
    )]
    pub revenue_tracker: Account<'info, RevenueTracker>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimRevenueAccountConstraints<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [ROYALTY_NFT_SEED, royalty_nft.mint.as_ref()],
        bump = royalty_nft.bump
    )]
    pub royalty_nft: Account<'info, RoyaltyNft>,
    
    #[account(
        seeds = [MASTER_NFT_SEED, master_nft.mint.as_ref()],
        bump = master_nft.bump,
        constraint = royalty_nft.master_nft == master_nft.key() @ CustomError::InvalidRoyaltySplit
    )]
    pub master_nft: Account<'info, MasterNft>,
    
    #[account(
        seeds = [ROYALTY_SPLIT_SEED, master_nft.key().as_ref()],
        bump = royalty_split.bump,
        constraint = royalty_split.master_nft == master_nft.key() @ CustomError::InvalidRoyaltySplit
    )]
    pub royalty_split: Account<'info, RoyaltySplit>,
    
    #[account(
        seeds = [TREASURY_SEED],
        bump = treasury.bump
    )]
    pub treasury: Account<'info, Treasury>,
    
    /// CHECK: Treasury wallet account
    #[account(
        mut,
        constraint = treasury_wallet.key() == treasury.treasury_wallet @ CustomError::Unauthorized
    )]
    pub treasury_wallet: UncheckedAccount<'info>,
    
    /// CHECK: Payer account for revenue distribution (program or exchange)
    #[account(mut)]
    pub payer: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}