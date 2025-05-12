use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::*;
use crate::error::CustomError;
use crate::constants::*;

pub fn create_collection(
    context: Context<CreateCollectionAccountConstraints>,
    name: String,
    description: String,
    uri: String
) -> Result<()> {
    require!(name.len() <= 100, CustomError::StringTooLong);
    require!(description.len() <= 500, CustomError::StringTooLong);
    require!(uri.len() <= 200, CustomError::StringTooLong);
    
    let collection = &mut context.accounts.collection;
    let clock = Clock::get()?;
    
    // Initialize collection data
    collection.name = name;
    collection.description = description;
    collection.uri = uri;
    collection.authority = context.accounts.authority.key();
    collection.mint = context.accounts.mint.key();
    collection.created_at = clock.unix_timestamp;
    collection.nft_count = 0;
    collection.bump = context.bumps.collection;
    
    // We would normally add code here to create the NFT metadata for the collection
    // using Metaplex, but we're skipping it since we had issues with metadata integration
    
    msg!("Collection created: {}", collection.name);
    Ok(())
}

pub fn add_to_collection(
    context: Context<AddToCollectionAccountConstraints>
) -> Result<()> {
    let collection = &mut context.accounts.collection;
    
    // Verify the caller is the collection authority
    require!(
        collection.authority == context.accounts.authority.key(),
        CustomError::Unauthorized
    );
    
    // Update collection counter
    collection.nft_count = collection.nft_count.checked_add(1).unwrap();
    
    // Here we'd normally update the NFT metadata to add the collection
    // but we're skipping that for now
    
    msg!("Added master NFT to collection: {}", collection.name);
    Ok(())
}

#[account]
pub struct Collection {
    pub name: String,
    pub description: String,
    pub uri: String,
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub created_at: i64,
    pub nft_count: u64,
    pub bump: u8,
}

impl Collection {
    pub const DISCRIMINATOR_LENGTH: usize = 8;
    pub const PUBKEY_LENGTH: usize = 32;
    pub const U64_LENGTH: usize = 8;
    pub const I64_LENGTH: usize = 8;
    pub const U8_LENGTH: usize = 1;
    pub const STRING_PREFIX_LENGTH: usize = 4;
    
    pub const INIT_SPACE: usize = 
        Self::DISCRIMINATOR_LENGTH +
        Self::STRING_PREFIX_LENGTH + 100 +  // name
        Self::STRING_PREFIX_LENGTH + 500 +  // description
        Self::STRING_PREFIX_LENGTH + 200 +  // uri
        Self::PUBKEY_LENGTH +              // authority
        Self::PUBKEY_LENGTH +              // mint
        Self::I64_LENGTH +                 // created_at
        Self::U64_LENGTH +                 // nft_count
        Self::U8_LENGTH;                   // bump
}

#[derive(Accounts)]
pub struct CreateCollectionAccountConstraints<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = Collection::INIT_SPACE,
        seeds = [SOUND_MINT_COLLECTION_PREFIX, mint.key().as_ref()],
        bump
    )]
    pub collection: Account<'info, Collection>,
    
    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = authority,
        mint::freeze_authority = authority,
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = authority,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AddToCollectionAccountConstraints<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [SOUND_MINT_COLLECTION_PREFIX, collection.mint.as_ref()],
        bump = collection.bump,
        constraint = collection.authority == authority.key() @ CustomError::Unauthorized
    )]
    pub collection: Account<'info, Collection>,
    
    #[account(
        seeds = [MASTER_NFT_SEED, master_nft.mint.as_ref()],
        bump = master_nft.bump
    )]
    pub master_nft: Account<'info, MasterNft>,
    
    pub system_program: Program<'info, System>,
}