use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3,
        mpl_token_metadata::types::DataV2,
        CreateMetadataAccountsV3,
        Metadata,
    },
    token::{Mint, Token, TokenAccount, mint_to, MintTo},
};
use crate::state::*;
use crate::error::CustomError;
use crate::constants::*;

pub fn mint_master_nft(
    context: Context<MintMasterNftAccountConstraints>,
    title: String,
    description: String,
    audio_uri: String,
    artwork_uri: String,
    metadata: Vec<MetadataItem>
) -> Result<()> {
    // Validate input lengths
    require!(title.len() <= 100, CustomError::StringTooLong);
    require!(description.len() <= 500, CustomError::StringTooLong);
    require!(audio_uri.len() <= 200, CustomError::StringTooLong);
    require!(artwork_uri.len() <= 200, CustomError::StringTooLong);
    require!(metadata.len() <= MAX_METADATA_ITEMS, CustomError::TooManyMetadataItems);
    
    // Validate metadata items
    for item in &metadata {
        require!(item.key.len() <= 50, CustomError::StringTooLong);
        require!(item.value.len() <= 50, CustomError::StringTooLong);
    }
    
    let clock = Clock::get()?;
    
    // Get accounts
    let master_nft = &mut context.accounts.master_nft;
    let artist_profile = &mut context.accounts.artist_profile;
    let treasury = &mut context.accounts.treasury;
    
    // Collect mint fee
    if treasury.mint_fee > 0 {
        require!(
            **context.accounts.authority.lamports.borrow() > treasury.mint_fee,
            CustomError::InsufficientFunds
        );
        
        // Transfer fee to treasury wallet
        let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
            &context.accounts.authority.key(),
            &treasury.treasury_wallet,
            treasury.mint_fee
        );
        
        anchor_lang::solana_program::program::invoke(
            &transfer_instruction,
            &[
                context.accounts.authority.to_account_info(),
                context.accounts.treasury_wallet.to_account_info(),
                context.accounts.system_program.to_account_info(),
            ]
        )?;
        
        treasury.total_revenue_collected = treasury.total_revenue_collected.checked_add(treasury.mint_fee).unwrap();
    }
    
    // Update artist profile
    artist_profile.track_count = artist_profile.track_count.checked_add(1).unwrap();
    
    // Initialize master NFT data
    master_nft.title = title;
    master_nft.description = description;
    master_nft.artist_profile = artist_profile.key();
    master_nft.audio_uri = audio_uri;
    master_nft.artwork_uri = artwork_uri;
    master_nft.metadata = metadata;
    master_nft.mint = context.accounts.mint.key();
    master_nft.is_transferable = true;
    master_nft.status = MasterNftStatus::Active;
    master_nft.created_at = clock.unix_timestamp;
    master_nft.bump = context.bumps.master_nft;
    
    // 1. Mint the NFT token
    msg!("Minting Token");
    mint_to(
        CpiContext::new(
            context.accounts.token_program.to_account_info(),
            MintTo {
                mint: context.accounts.mint.to_account_info(),
                to: context.accounts.token_account.to_account_info(),
                authority: context.accounts.authority.to_account_info(),
            },
        ),
        1, // Mint exactly 1 token
    )?;
    
    // 2. Create NFT metadata using Metaplex
    msg!("Creating metadata account");
    let metadata_title = master_nft.title.clone();
    let metadata_symbol = "SNDM".to_string();
    
    // Prepare the NFT URI with audio URI as animation_url
    let nft_uri = format!("{{\"name\":\"{}\",\"description\":\"{}\",\"image\":\"{}\",\"animation_url\":\"{}\"}}",
        metadata_title,
        master_nft.description,
        master_nft.artwork_uri,
        master_nft.audio_uri
    );
    
    // Create metadata using Metaplex functions with DataV2 struct
    create_metadata_accounts_v3(
        CpiContext::new(
            context.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: context.accounts.metadata_account.to_account_info(),
                mint: context.accounts.mint.to_account_info(),
                mint_authority: context.accounts.authority.to_account_info(),
                payer: context.accounts.authority.to_account_info(),
                update_authority: context.accounts.authority.to_account_info(),
                system_program: context.accounts.system_program.to_account_info(),
                rent: context.accounts.rent.to_account_info(),
            },
        ),
        DataV2 {
            name: metadata_title,
            symbol: metadata_symbol,
            uri: nft_uri,
            seller_fee_basis_points: 100, // 1% royalty
            creators: Some(vec![
                anchor_spl::metadata::mpl_token_metadata::types::Creator {
                    address: context.accounts.authority.key(),
                    verified: true,
                    share: 100,
                }
            ]),
            collection: None,
            uses: None,
        },
        true,  // Is mutable
        true,  // Update authority is signer
        None,  // Collection details
    )?;
    
    msg!("Master NFT created: {}", master_nft.title);
    Ok(())
}

pub fn update_master_nft(
    context: Context<UpdateMasterNftAccountConstraints>,
    description: Option<String>,
    metadata: Option<Vec<MetadataItem>>,
    is_transferable: Option<bool>,
    status: Option<MasterNftStatus>
) -> Result<()> {
    let master_nft = &mut context.accounts.master_nft;
    
    if let Some(new_description) = description {
        require!(new_description.len() <= 500, CustomError::StringTooLong);
        master_nft.description = new_description;
    }
    
    if let Some(new_metadata) = metadata {
        require!(new_metadata.len() <= MAX_METADATA_ITEMS, CustomError::TooManyMetadataItems);
        
        // Validate metadata items
        for item in &new_metadata {
            require!(item.key.len() <= 50, CustomError::StringTooLong);
            require!(item.value.len() <= 50, CustomError::StringTooLong);
        }
        
        master_nft.metadata = new_metadata;
    }
    
    if let Some(new_is_transferable) = is_transferable {
        master_nft.is_transferable = new_is_transferable;
    }
    
    if let Some(new_status) = status {
        master_nft.status = new_status;
    }
    
    msg!("Master NFT updated: {}", master_nft.title);
    Ok(())
}

#[derive(Accounts)]
pub struct MintMasterNftAccountConstraints<'info> {
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
        init,
        payer = authority,
        space = MasterNft::INIT_SPACE,
        seeds = [MASTER_NFT_SEED, mint.key().as_ref()],
        bump
    )]
    pub master_nft: Account<'info, MasterNft>,
    
    // NFT mint account
    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = authority.key(),
        mint::freeze_authority = authority.key(),
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
    
    // Metadata account for the NFT
    /// CHECK: Metadata account is checked by the token metadata program
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        seeds::program = token_metadata_program.key()
    )]
    pub metadata_account: UncheckedAccount<'info>,
    
    // Treasury account to charge fees
    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,
    
    /// CHECK: Treasury wallet account
    #[account(
        mut,
        constraint = treasury_wallet.key() == treasury.treasury_wallet @ CustomError::Unauthorized
    )]
    pub treasury_wallet: UncheckedAccount<'info>,
    
    // Required programs
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    // pub token_metadata_program: Program<'info, Metadata>,
        /// CHECK: This is the Metaplex token metadata program
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateMasterNftAccountConstraints<'info> {
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
        mut,
        seeds = [MASTER_NFT_SEED, master_nft.mint.as_ref()],
        bump = master_nft.bump,
        constraint = master_nft.artist_profile == artist_profile.key() @ CustomError::Unauthorized
    )]
    pub master_nft: Account<'info, MasterNft>,
    
    pub system_program: Program<'info, System>,
}