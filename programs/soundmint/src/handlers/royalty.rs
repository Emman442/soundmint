use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::*;
use crate::error::CustomError;
use crate::constants::*;

pub fn create_royalty_split(
    context: Context<CreateRoyaltySplitAccountConstraints>,
    collaborators: Vec<Collaborator>
) -> Result<()> {
    // Validate collaborators
    require!(!collaborators.is_empty(), CustomError::InvalidRoyaltySplit);
    require!(collaborators.len() <= 10, CustomError::TooManyCollaborators);
    
    // Calculate total shares
    let mut total_basis_points = 0_u16;
    for collaborator in &collaborators {
        require!(collaborator.share_basis_points > 0, CustomError::InvalidRoyaltyShares);
        require!(collaborator.name.len() <= 50, CustomError::StringTooLong);
        total_basis_points = total_basis_points.checked_add(collaborator.share_basis_points).unwrap();
    }
    
    // Ensure total is 100%
    require!(total_basis_points == TOTAL_BASIS_POINTS, CustomError::InvalidRoyaltyShares);
    
    // Initialize royalty split
    let royalty_split = &mut context.accounts.royalty_split;
    let clock = Clock::get()?;
    
    royalty_split.master_nft = context.accounts.master_nft.key();
    royalty_split.collaborators = collaborators;
    royalty_split.total_basis_points = total_basis_points;
    royalty_split.total_revenue_collected = 0;
    royalty_split.created_at = clock.unix_timestamp;
    royalty_split.last_revenue_timestamp = 0;
    royalty_split.bump = context.bumps.royalty_split;
    
    msg!("Royalty split created for track: {}", context.accounts.master_nft.title);
    Ok(())
}

pub fn mint_royalty_nft(
    context: Context<MintRoyaltyNftAccountConstraints>,
    share_basis_points: u16
) -> Result<()> {
    let royalty_split = &mut context.accounts.royalty_split;
    
    // Validate share allocation doesn't exceed remaining shares
    let mut allocated_share = 0_u16;
    for collaborator in &royalty_split.collaborators {
        if collaborator.address == context.accounts.authority.key() {
            allocated_share = allocated_share.checked_add(collaborator.share_basis_points).unwrap();
        }
    }
    
    require!(share_basis_points <= allocated_share, CustomError::InvalidRoyaltyShares);
    
    // Create royalty NFT
    let royalty_nft = &mut context.accounts.royalty_nft;
    let clock = Clock::get()?;
    
    royalty_nft.master_nft = context.accounts.master_nft.key();
    royalty_nft.mint = context.accounts.mint.key();
    royalty_nft.share_basis_points = share_basis_points;
    royalty_nft.amount_claimed = 0;
    royalty_nft.last_claimed_at = 0;
    royalty_nft.created_at = clock.unix_timestamp;
    royalty_nft.bump = context.bumps.royalty_nft;
    
    msg!("Royalty NFT minted with {}% share", share_basis_points as f32 / 100.0);
    Ok(())
}

#[derive(Accounts)]
pub struct CreateRoyaltySplitAccountConstraints<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [ARTIST_PROFILE_SEED, authority.key().as_ref()],
        bump,
        constraint = artist_profile.authority == authority.key() @ CustomError::Unauthorized
    )]
    pub artist_profile: Account<'info, ArtistProfile>,
    
    #[account(
        seeds = [MASTER_NFT_SEED, master_nft.mint.as_ref()],
        bump = master_nft.bump,
        constraint = master_nft.artist_profile == artist_profile.key() @ CustomError::Unauthorized
    )]
    pub master_nft: Account<'info, MasterNft>,
    
    #[account(
        init,
        payer = authority,
        space = RoyaltySplit::INIT_SPACE,
        seeds = [ROYALTY_SPLIT_SEED, master_nft.key().as_ref()],
        bump
    )]
    pub royalty_split: Account<'info, RoyaltySplit>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintRoyaltyNftAccountConstraints<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        seeds = [MASTER_NFT_SEED, master_nft.mint.as_ref()],
        bump = master_nft.bump
    )]
    pub master_nft: Box<Account<'info, MasterNft>>,
    
    #[account(
        mut,
        seeds = [ROYALTY_SPLIT_SEED, master_nft.key().as_ref()],
        bump = royalty_split.bump,
        constraint = royalty_split.master_nft == master_nft.key() @ CustomError::InvalidRoyaltySplit
    )]
    pub royalty_split: Box<Account<'info, RoyaltySplit>>,
    
    // NFT mint account
    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = authority,
        mint::freeze_authority = authority,
    )]
    pub mint: Account<'info, Mint>,
    
    // Associated token account for the NFT
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = authority,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = authority,
        space = RoyaltyNft::INIT_SPACE,
        seeds = [ROYALTY_NFT_SEED, mint.key().as_ref()],
        bump
    )]
    pub royalty_nft: Box<Account<'info, RoyaltyNft>>,
    
    // Required programs
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}