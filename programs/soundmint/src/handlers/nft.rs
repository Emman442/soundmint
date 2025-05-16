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
    mut context: Context<MintMasterNftAccountConstraints>,
    title: String,
    description: String,
    audio_uri: String,
    artwork_uri: String,
    metadata: Vec<MetadataItem>,
) -> Result<()> {
    validate_inputs(&title, &description, &audio_uri, &artwork_uri, &metadata)?;
    collect_mint_fee(&mut context)?;
    update_artist_profile(&mut context.accounts.artist_profile)?;
    initialize_master_nft(
        &mut context.accounts.master_nft,
        context.accounts.artist_profile.key(),
        context.accounts.mint.key(),
        context.bumps.master_nft,
        title,
        description,
        audio_uri,
        artwork_uri,
        metadata, // Added metadata parameter
    )?;
    mint_nft_token(&context)?;

    msg!("Master NFT minted, metadata creation required");
    Ok(())
}

pub fn create_nft_metadata(
    context: Context<CreateMetadataAccountConstraints>,
) -> Result<()> {
    create_nft_metadata_account(&context)?;
    msg!("Metadata created for Master NFT: {}", context.accounts.master_nft.title);
    Ok(())
}

pub fn update_master_nft(
    context: Context<UpdateMasterNftAccountConstraints>,
    description: Option<String>,
    metadata: Option<Vec<MetadataItem>>,
    is_transferable: Option<bool>,
    status: Option<MasterNftStatus>,
) -> Result<()> {
    let master_nft = &mut context.accounts.master_nft;

    if let Some(new_description) = description {
        require!(new_description.len() <= 500, CustomError::StringTooLong);
        master_nft.description = new_description;
    }

    if let Some(new_metadata) = metadata {
        require!(
            new_metadata.len() <= MAX_METADATA_ITEMS,
            CustomError::TooManyMetadataItems
        );

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

// Helper Functions
fn validate_inputs(
    title: &str,
    description: &str,
    audio_uri: &str,
    artwork_uri: &str,
    metadata: &[MetadataItem],
) -> Result<()> {
    require!(title.len() <= 100, CustomError::StringTooLong);
    require!(description.len() <= 500, CustomError::StringTooLong);
    require!(audio_uri.len() <= 200, CustomError::StringTooLong);
    require!(artwork_uri.len() <= 200, CustomError::StringTooLong);
    require!(metadata.len() <= MAX_METADATA_ITEMS, CustomError::TooManyMetadataItems);

    for item in metadata {
        require!(item.key.len() <= 50, CustomError::StringTooLong);
        require!(item.value.len() <= 50, CustomError::StringTooLong);
    }

    Ok(())
}

fn collect_mint_fee(context: &mut Context<MintMasterNftAccountConstraints>) -> Result<()> {
    let treasury = &mut context.accounts.treasury;
    if treasury.mint_fee > 0 {
        require!(
            **context.accounts.authority.lamports.borrow() > treasury.mint_fee,
            CustomError::InsufficientFunds
        );

        let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
            &context.accounts.authority.key(),
            &treasury.treasury_wallet,
            treasury.mint_fee,
        );

        anchor_lang::solana_program::program::invoke(
            &transfer_instruction,
            &[
                context.accounts.authority.to_account_info(),
                context.accounts.treasury_wallet.to_account_info(),
                context.accounts.system_program.to_account_info(),
            ],
        )?;

        treasury.total_revenue_collected = treasury
            .total_revenue_collected
            .checked_add(treasury.mint_fee)
            .unwrap();
    }
    Ok(())
}

fn update_artist_profile<'info>(artist_profile: &mut Account<'info, ArtistProfile>) -> Result<()> {
    artist_profile.track_count = artist_profile.track_count.checked_add(1).unwrap();
    Ok(())
}

fn initialize_master_nft<'info>(
    master_nft: &mut Account<'info, MasterNft>,
    artist_profile_key: Pubkey,
    mint_key: Pubkey,
    bump: u8,
    title: String,
    description: String,
    audio_uri: String,
    artwork_uri: String,
    metadata: Vec<MetadataItem>,
) -> Result<()> {
    let clock = Clock::get()?;
    master_nft.title = title;
    master_nft.description = description;
    master_nft.artist_profile = artist_profile_key;
    master_nft.audio_uri = audio_uri;
    master_nft.artwork_uri = artwork_uri;
    master_nft.metadata = metadata;
    master_nft.mint = mint_key;
    master_nft.is_transferable = true;
    master_nft.status = MasterNftStatus::Active;
    master_nft.created_at = clock.unix_timestamp;
    master_nft.bump = bump;
    Ok(())
}

fn mint_nft_token(context: &Context<MintMasterNftAccountConstraints>) -> Result<()> {
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
        1,
    )?;
    Ok(())
}

fn create_nft_uri(
    title: &str,
    description: &str,
    artwork_uri: &str,
    audio_uri: &str,
) -> String {
    format!(
        "{{\"name\":\"{}\",\"description\":\"{}\",\"image\":\"{}\",\"animation_url\":\"{}\"}}",
        title, description, artwork_uri, audio_uri
    )
}

fn create_nft_metadata_account(context: &Context<CreateMetadataAccountConstraints>) -> Result<()> {
    msg!("Creating metadata account");
    let metadata_title = context.accounts.master_nft.title.clone();
    let metadata_symbol = "SNDM".to_string();

    let nft_uri = create_nft_uri(
        &metadata_title,
        &context.accounts.master_nft.description,
        &context.accounts.master_nft.artwork_uri,
        &context.accounts.master_nft.audio_uri,
    );

    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
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
            &[&[
                b"metadata",
                context.accounts.token_metadata_program.key().as_ref(),
                context.accounts.mint.key().as_ref(),
                &[context.bumps.metadata_account],
            ]],
        ),
        DataV2 {
            name: metadata_title,
            symbol: metadata_symbol,
            uri: nft_uri,
            seller_fee_basis_points: 100,
            creators: Some(vec![anchor_spl::metadata::mpl_token_metadata::types::Creator {
                address: context.accounts.authority.key(),
                verified: true,
                share: 100,
            }]),
            collection: None,
            uses: None,
        },
        true,
        true,
        None,
    )?;
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
    pub artist_profile: Box<Account<'info, ArtistProfile>>,

    #[account(
        init,
        payer = authority,
        space = MasterNft::INIT_SPACE,
        seeds = [MASTER_NFT_SEED, mint.key().as_ref()],
        bump
    )]
    pub master_nft: Box<Account<'info, MasterNft>>,

    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = authority.key(),
        mint::freeze_authority = authority.key(),
    )]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = authority,
    )]
    pub token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury.bump,
    )]
    pub treasury: Box<Account<'info, Treasury>>,

    /// CHECK: Treasury wallet account
    #[account(
        mut,
        constraint = treasury_wallet.key() == treasury.treasury_wallet @ CustomError::Unauthorized
    )]
    pub treasury_wallet: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateMetadataAccountConstraints<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [MASTER_NFT_SEED, mint.key().as_ref()],
        bump = master_nft.bump,
        constraint = master_nft.mint == mint.key() @ CustomError::Unauthorized
    )]
    pub master_nft: Box<Account<'info, MasterNft>>,

    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    /// CHECK: Metadata account is checked by the token metadata program
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump
    )]
    pub metadata_account: UncheckedAccount<'info>,

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
    pub artist_profile: Box<Account<'info, ArtistProfile>>,

    #[account(
        mut,
        seeds = [MASTER_NFT_SEED, master_nft.mint.as_ref()],
        bump = master_nft.bump,
        constraint = master_nft.artist_profile == artist_profile.key() @ CustomError::Unauthorized
    )]
    pub master_nft: Box<Account<'info, MasterNft>>,

    pub system_program: Program<'info, System>,
}