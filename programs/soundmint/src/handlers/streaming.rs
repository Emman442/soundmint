use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::CustomError;
use crate::constants::*;

pub fn register_streaming_batch(
    context: Context<RegisterStreamingBatchAccountConstraints>,
    streaming_data: Vec<StreamingData>
) -> Result<()> {
    require!(!streaming_data.is_empty(), CustomError::InvalidData);
    require!(streaming_data.len() <= MAX_STREAMING_BATCH_SIZE, CustomError::BatchTooLarge);
    
    let clock = Clock::get()?;
    let treasury = &mut context.accounts.treasury;
    
    // Only authorized streaming provider can register streaming data
    require!(
        context.accounts.streaming_provider.key() == treasury.streaming_provider,
        CustomError::Unauthorized
    );
    
    let mut total_platform_fee : u64 = 0;
    
    // Process each streaming record
    for record in &streaming_data {
        require!(record.amount > 0, CustomError::InvalidAmount);
        
        // Get revenue tracker for this master NFT
        let seeds = &[
            REVENUE_TRACKER_SEED,
            record.master_nft.as_ref(),
            &[record.bump]
        ];
        
        // Create CPI context for revenue tracker
        let revenue_tracker_account_info = context.remaining_accounts.iter().find(|account| {
            account.key() == Pubkey::find_program_address(
                &[REVENUE_TRACKER_SEED, record.master_nft.as_ref()],
                &crate::ID
            ).0
        });
        
        let royalty_split_account_info = context.remaining_accounts.iter().find(|account| {
            account.key() == Pubkey::find_program_address(
                &[ROYALTY_SPLIT_SEED, record.master_nft.as_ref()],
                &crate::ID
            ).0
        });
        
        if let (Some(revenue_tracker_info), Some(royalty_split_info)) = (revenue_tracker_account_info, royalty_split_account_info) {
            // Calculate platform fee
            let platform_fee: u64 = record.amount
                .checked_mul(treasury.platform_fee_basis_points as u64).unwrap()
                .checked_div(TOTAL_BASIS_POINTS as u64).unwrap();
            
            total_platform_fee = total_platform_fee.checked_add(platform_fee).unwrap();
            
            // Update revenue tracker
            let mut revenue_tracker_data = revenue_tracker_info.try_borrow_mut_data()?;
            let revenue_tracker = RevenueTracker::try_deserialize(&mut &revenue_tracker_data[..])?;
            
            // Get mutable reference to the revenue tracker
            let mut revenue_tracker = RevenueTracker {
                master_nft: revenue_tracker.master_nft,
                total_revenue: revenue_tracker.total_revenue.checked_add(record.amount).unwrap(),
                streaming_revenue: revenue_tracker.streaming_revenue.checked_add(record.amount).unwrap(),
                sales_revenue: revenue_tracker.sales_revenue,
                other_revenue: revenue_tracker.other_revenue,
                transactions: revenue_tracker.transactions.clone(),
                created_at: revenue_tracker.created_at,
                last_revenue_timestamp: clock.unix_timestamp,
                bump: revenue_tracker.bump,
            };
            
            // Add transaction record
            if revenue_tracker.transactions.len() < RevenueTracker::MAX_TRANSACTIONS {
                revenue_tracker.transactions.push(RevenueTransaction {
                    amount: record.amount,
                    source: "streaming".to_string(),
                    description: format!("Batch streaming revenue from {}", context.accounts.streaming_provider.key()),
                    timestamp: clock.unix_timestamp,
                });
            }
            
            // Reserialize the revenue tracker
            revenue_tracker.try_serialize(&mut *revenue_tracker_data)?;
            
            // Update royalty split
            let mut royalty_split_data = royalty_split_info.try_borrow_mut_data()?;
            let royalty_split = RoyaltySplit::try_deserialize(&mut &royalty_split_data[..])?;
            
            // Get mutable reference to the royalty split
            let mut royalty_split = RoyaltySplit {
                master_nft: royalty_split.master_nft,
                collaborators: royalty_split.collaborators.clone(),
                total_basis_points: royalty_split.total_basis_points,
                total_revenue_collected: royalty_split.total_revenue_collected.checked_add(record.amount).unwrap(),
                created_at: royalty_split.created_at,
                last_revenue_timestamp: clock.unix_timestamp,
                bump: royalty_split.bump,
            };
            
            // Reserialize the royalty split
            royalty_split.try_serialize(&mut *royalty_split_data)?;
        }
    }
    
    // Transfer platform fees to treasury
    if total_platform_fee > 0 {
        let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
            &context.accounts.streaming_provider.key(),
            &treasury.treasury_wallet,
            total_platform_fee
        );
        
        anchor_lang::solana_program::program::invoke(
            &transfer_instruction,
            &[
                context.accounts.streaming_provider.to_account_info(),
                context.accounts.treasury_wallet.to_account_info(),
                context.accounts.system_program.to_account_info(),
            ]
        )?;
        
        // Update treasury revenue
        treasury.total_revenue_collected = treasury.total_revenue_collected.checked_add(total_platform_fee).unwrap();
    }
    
    msg!("Processed streaming batch with {} records", streaming_data.len());
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StreamingData {
    pub master_nft: Pubkey,
    pub amount: u64,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct RegisterStreamingBatchAccountConstraints<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub streaming_provider: Signer<'info>,
    
    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury.bump
    )]
    pub treasury: Account<'info, Treasury>,
    
    /// CHECK: Treasury wallet for receiving fees
    #[account(
        mut,
        constraint = treasury_wallet.key() == treasury.treasury_wallet @ CustomError::InvalidTreasuryWallet
    )]
    pub treasury_wallet: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}