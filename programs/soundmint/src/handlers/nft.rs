use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_token_metadata::instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs};
use mpl_token_metadata::types::{Creator, DataV2};
use anchor_spl::associated_token::AssociatedToken;
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
    
    // Create NFT metadata using Metaplex
    let metadata_title = master_nft.title.clone();
    let metadata_symbol = "SNDM".to_string();
    let creator = vec![
        anchor_spl::metadata::Creator {
            address: context.accounts.authority.key(),
            verified: true,
            share: 100,
        }
    ];
    
    let metadata_uri_json = format!("{{\"name\":\"{}\",\"description\":\"{}\",\"image\":\"{}\",\"animation_url\":\"{}\"}}",
        metadata_title,
        master_nft.description,
        master_nft.artwork_uri,
        master_nft.audio_uri
    );
    
    // Create metadata using Metaplex functions
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
        metadata_uri_json,
        metadata_symbol,
        creator,
        100, // Royalty basis points - 1%
        true,
        true,
        None,
        None,
        None,
    )?;
    
    msg!("Master NFT created: {}", master_nft.title);
    Ok(())
}

// use anchor_lang::prelude::*;
// use anchor_spl::token::{Mint, Token, TokenAccount};
// use anchor_spl::associated_token::AssociatedToken;
// use mpl_token_metadata::instruction as token_metadata_instruction;
// use mpl_token_metadata::state::Creator;
// use crate::state::*;
// use crate::error::CustomError;
// use crate::constants::*;

// pub fn mint_master_nft(
//     context: Context<MintMasterNftAccountConstraints>,
//     title: String,
//     description: String,
//     audio_uri: String,
//     artwork_uri: String,
//     metadata: Vec<MetadataItem>
// ) -> Result<()> {
//     // Validate input lengths
//     require!(title.len() <= 100, CustomError::StringTooLong);
//     require!(description.len() <= 500, CustomError::StringTooLong);
//     require!(audio_uri.len() <= 200, CustomError::StringTooLong);
//     require!(artwork_uri.len() <= 200, CustomError::StringTooLong);
//     require!(metadata.len() <= MAX_METADATA_ITEMS, CustomError::TooManyMetadataItems);
    
//     // Validate metadata items
//     for item in &metadata {
//         require!(item.key.len() <= 50, CustomError::StringTooLong);
//         require!(item.value.len() <= 50, CustomError::StringTooLong);
//     }
    
//     let clock = Clock::get()?;
    
//     // Get accounts
//     let master_nft = &mut context.accounts.master_nft;
//     let artist_profile = &mut context.accounts.artist_profile;
//     let treasury = &mut context.accounts.treasury;
    
//     // Collect mint fee
//     if treasury.mint_fee > 0 {
//         require!(
//             **context.accounts.authority.lamports.borrow() > treasury.mint_fee,
//             CustomError::InsufficientFunds
//         );
        
//         // Transfer fee to treasury wallet
//         let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
//             &context.accounts.authority.key(),
//             &treasury.treasury_wallet,
//             treasury.mint_fee
//         );
        
//         anchor_lang::solana_program::program::invoke(
//             &transfer_instruction,
//             &[
//                 context.accounts.authority.to_account_info(),
//                 context.accounts.treasury_wallet.to_account_info(),
//                 context.accounts.system_program.to_account_info(),
//             ]
//         )?;
        
//         treasury.total_revenue_collected = treasury.total_revenue_collected.checked_add(treasury.mint_fee).unwrap();
//     }
    
//     // Update artist profile
//     artist_profile.track_count = artist_profile.track_count.checked_add(1).unwrap();
    
//     // Initialize master NFT data
//     master_nft.title = title;
//     master_nft.description = description;
//     master_nft.artist_profile = artist_profile.key();
//     master_nft.audio_uri = audio_uri;
//     master_nft.artwork_uri = artwork_uri;
//     master_nft.metadata = metadata;
//     master_nft.mint = context.accounts.mint.key();
//     master_nft.is_transferable = true;
//     master_nft.status = MasterNftStatus::Active;
//     master_nft.created_at = clock.unix_timestamp;
//     master_nft.bump = context.bumps.master_nft;
    
//     // Create NFT metadata using Metaplex
//     let metadata_title = master_nft.title.clone();
//     let metadata_symbol = "SNDM".to_string();
    
//     // Create metadata URI JSON with audio
//     let metadata_uri = format!("{{\"name\":\"{}\",\"description\":\"{}\",\"image\":\"{}\",\"animation_url\":\"{}\"}}",
//         metadata_title,
//         master_nft.description,
//         master_nft.artwork_uri,
//         master_nft.audio_uri
//     );

//     // Use direct instruction creation for metadata
//     let creators = vec![
//         mpl_token_metadata::state::Creator {
//             address: context.accounts.authority.key(),
//             verified: true, 
//             share: 100,
//         },
//     ];

//     // Create the metadata creation instruction
//     let create_metadata_ix = mpl_token_metadata::instruction::create_metadata_accounts_v3(
//         context.accounts.token_metadata_program.key(),   // Program ID
//         context.accounts.metadata_account.key(),         // Metadata account
//         context.accounts.mint.key(),                     // Mint account
//         context.accounts.authority.key(),                // Mint authority
//         context.accounts.authority.key(),                // Payer
//         context.accounts.authority.key(),                // Update authority
//         metadata_title,                                  // Name
//         metadata_symbol,                                 // Symbol
//         metadata_uri,                                    // URI
//         Some(creators),                                  // Creators
//         100,                                             // Seller fee basis points (1%)
//         true,                                            // Update authority is signer
//         true,                                            // Is mutable
//         None,                                            // Collection
//         None,                                            // Uses
//         None,                                            // Collection Details
//     );
    
//     // Invoke the instruction
//     anchor_lang::solana_program::program::invoke(
//         &create_metadata_ix,
//         &[
//             context.accounts.metadata_account.to_account_info(),
//             context.accounts.mint.to_account_info(),
//             context.accounts.authority.to_account_info(),
//             context.accounts.authority.to_account_info(),
//             context.accounts.authority.to_account_info(),
//             context.accounts.system_program.to_account_info(),
//             context.accounts.rent.to_account_info(),
//         ],
//     )?;
    
//     msg!("Master NFT created: {}", master_nft.title);
//     Ok(())
// }


// use anchor_lang::prelude::*;
// use anchor_spl::token::{Mint, Token, TokenAccount};
// // use anchor_spl::associated_token::AssociatedToken;
// use anchor_spl::{
//     associated_token::AssociatedToken,
//     metadata::{
//         Metadata,
//         MetadataAccount,
//         MasterEditionAccount,
//     },
//     token_interface::{
//         TokenInterface,
//         Mint,
//         TokenAccount,
//         TransferChecked,
//         transfer_checked,
//     }
// };
// use mpl_token_metadata::instructions as token_metadata_instructions;
// use crate::state::{
//     MasterNft,
//     ArtistProfile,
//     Treasury,
//     MetadataItem,
//     MasterNftStatus,
// };
// use crate::error::CustomError;
// use crate::constants::*;

// pub fn mint_master_nft(
//     context: Context<MintMasterNftAccountConstraints>,
//     title: String,
//     description: String,
//     audio_uri: String,
//     artwork_uri: String,
//     metadata: Vec<MetadataItem>
// ) -> Result<()> {
//     // Validate input lengths
//     require!(title.len() <= 100, CustomError::StringTooLong);
//     require!(description.len() <= 500, CustomError::StringTooLong);
//     require!(audio_uri.len() <= 200, CustomError::StringTooLong);
//     require!(artwork_uri.len() <= 200, CustomError::StringTooLong);
//     require!(metadata.len() <= MAX_METADATA_ITEMS, CustomError::TooManyMetadataItems);
    
//     // Validate metadata items
//     for item in &metadata {
//         require!(item.key.len() <= 50, CustomError::StringTooLong);
//         require!(item.value.len() <= 50, CustomError::StringTooLong);
//     }
    
//     let clock = Clock::get()?;
    
//     // Get accounts
//     let master_nft = &mut context.accounts.master_nft;
//     let artist_profile = &mut context.accounts.artist_profile;
//     let treasury = &mut context.accounts.treasury;
    
//     // Collect mint fee
//     if treasury.mint_fee > 0 {
//         require!(
//             **context.accounts.authority.lamports.borrow() > treasury.mint_fee,
//             CustomError::InsufficientFunds
//         );
        
//         // Transfer fee to treasury wallet
//         let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
//             &context.accounts.authority.key(),
//             &treasury.treasury_wallet,
//             treasury.mint_fee
//         );
        
//         anchor_lang::solana_program::program::invoke(
//             &transfer_instruction,
//             &[
//                 context.accounts.authority.to_account_info(),
//                 context.accounts.treasury_wallet.to_account_info(),
//                 context.accounts.system_program.to_account_info(),
//             ]
//         )?;
        
//         treasury.total_revenue_collected = treasury.total_revenue_collected.checked_add(treasury.mint_fee).unwrap();
//     }
    
//     // Update artist profile
//     artist_profile.track_count = artist_profile.track_count.checked_add(1).unwrap();
    
//     // Initialize master NFT data
//     master_nft.title = title;
//     master_nft.description = description;
//     master_nft.artist_profile = artist_profile.key();
//     master_nft.audio_uri = audio_uri;
//     master_nft.artwork_uri = artwork_uri;
//     master_nft.metadata = metadata;
//     master_nft.mint = context.accounts.mint.key();
//     master_nft.is_transferable = true;
//     master_nft.status = MasterNftStatus::Active;
//     master_nft.created_at = clock.unix_timestamp;
//     master_nft.bump = context.bumps.master_nft;
    
//     // Create NFT metadata using Metaplex
//     let metadata_title = master_nft.title.clone();
//     let metadata_symbol = "SNDM".to_string();
    
//     // Create metadata URI JSON with audio
//     let metadata_uri = format!("{{\"name\":\"{}\",\"description\":\"{}\",\"image\":\"{}\",\"animation_url\":\"{}\"}}",
//         metadata_title,
//         master_nft.description,
//         master_nft.artwork_uri,
//         master_nft.audio_uri
//     );

//     // Create the creators array using the new Metaplex v5 structure
//     let creators = vec![
//         token_metadata_instructions::CreateV1CpiBuilder::new()
//             .creator(context.accounts.authority.key(), true, 100)
//             .to_creator(),
//     ];

//     // Create the metadata creation instruction using the MPL Token Metadata v5.1.0 builder
//     let create_metadata_ix = token_metadata_instructions::CreateV1CpiBuilder::new()
//         .metadata(context.accounts.metadata_account.key())
//         .mint(context.accounts.mint.key())
//         .authority(context.accounts.authority.key())
//         .payer(context.accounts.authority.key())
//         .update_authority(context.accounts.authority.key(), true)
//         .name(metadata_title)
//         .symbol(metadata_symbol)
//         .uri(metadata_uri)
//         .creators(creators)
//         .seller_fee_basis_points(100) // 1%
//         .collection_details_toggle(false)
//         .build()
//         .instruction();
    
//     // Invoke the instruction
//     anchor_lang::solana_program::program::invoke(
//         &create_metadata_ix,
//         &[
//             context.accounts.metadata_account.to_account_info(),
//             context.accounts.mint.to_account_info(),
//             context.accounts.authority.to_account_info(),
//             context.accounts.authority.to_account_info(),
//             context.accounts.authority.to_account_info(),
//             context.accounts.token_metadata_program.to_account_info(),
//             context.accounts.system_program.to_account_info(),
//             context.accounts.rent.to_account_info(),
//         ],
//     )?;
    
//     msg!("Master NFT created: {}", master_nft.title);
//     Ok(())
// }

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
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,
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