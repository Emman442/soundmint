pub mod constants;
pub mod error;
pub mod handlers;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use handlers::*;
pub use state::*;

declare_id!("XsRj5cL2yV7BQehEsJHSEoy3P2Y4YXpogv4E2CJrzYQ");

#[program]
pub mod soundmint {
    use super::*;

    pub fn initialize(context: Context<Initialize>) -> Result<()> {
        initialize::handler(context)
    }
    
    pub fn create_artist_profile(
        context: Context<CreateArtistProfileAccountConstraints>,
        name: String,
        description: String, 
        profile_image_uri: String
    ) -> Result<()> {
        artist::create_artist_profile(context, name, description, profile_image_uri)
    }
    
    pub fn update_artist_profile(
        context: Context<UpdateArtistProfileAccountConstraints>,
        name: Option<String>,
        description: Option<String>,
        profile_image_uri: Option<String>,
        social_links: Option<Vec<SocialLink>>
    ) -> Result<()> {
        artist::update_artist_profile(context, name, description, profile_image_uri, social_links)
    }

        pub fn mint_master_nft(
        context: Context<MintMasterNftAccountConstraints>,
        title: String,
        description: String,
        audio_uri: String,
        artwork_uri: String,
        metadata: Vec<MetadataItem>
    ) -> Result<()> {
        nft::mint_master_nft(context, title, description, audio_uri, artwork_uri, metadata)
    }
    
    pub fn update_master_nft(
        context: Context<UpdateMasterNftAccountConstraints>,
        description: Option<String>,
        metadata: Option<Vec<MetadataItem>>,
        is_transferable: Option<bool>,
        status: Option<MasterNftStatus>
    ) -> Result<()> {
        nft::update_master_nft(context, description, metadata, is_transferable, status)
    }
    
    pub fn create_royalty_split(
        context: Context<CreateRoyaltySplitAccountConstraints>,
        collaborators: Vec<Collaborator>
    ) -> Result<()> {
        royalty::create_royalty_split(context, collaborators)
    }
    
    pub fn mint_royalty_nft(
        context: Context<MintRoyaltyNftAccountConstraints>,
        share_basis_points: u16
    ) -> Result<()> {
        royalty::mint_royalty_nft(context, share_basis_points)
    }

    pub fn track_revenue(
        context: Context<TrackRevenueAccountConstraints>,
        amount: u64,
        source: String,
        description: String
    ) -> Result<()> {
        revenue::track_revenue(context, amount, source, description)
    }

    pub fn claim_revenue(
        context: Context<ClaimRevenueAccountConstraints>
    ) -> Result<()> {
        revenue::claim_revenue(context)
    }

    pub fn update_treasury_config(
        context: Context<UpdateTreasuryConfigAccountConstraints>,
        mint_fee: Option<u64>,
        platform_fee_basis_points: Option<u16>,
        new_treasury_wallet: Option<Pubkey>
    ) -> Result<()> {
        admin::treasury::update_treasury_config(context, mint_fee, platform_fee_basis_points, new_treasury_wallet)
    }

    pub fn update_streaming_provider(
        context: Context<UpdateTreasuryConfigAccountConstraints>,
        new_streaming_provider: Pubkey
    ) -> Result<()> {
        admin::treasury::update_streaming_provider(context, new_streaming_provider)
    }

    pub fn withdraw_treasury_funds(
        context: Context<WithdrawTreasuryFundsAccountConstraints>,
        amount: u64
    ) -> Result<()> {
        admin::treasury::withdraw_treasury_funds(context, amount)
    }

    pub fn verify_artist(
        context: Context<VerifyArtistAccountConstraints>,
        verify: bool
    ) -> Result<()> {
        admin::artist::verify_artist(context, verify)
    }

    pub fn register_streaming_batch(
        context: Context<RegisterStreamingBatchAccountConstraints>,
        streaming_data: Vec<StreamingData>
    ) -> Result<()> {
        streaming::register_streaming_batch(context, streaming_data)
    }

    pub fn create_collection(
        context: Context<CreateCollectionAccountConstraints>,
        name: String,
        description: String,
        uri: String
    ) -> Result<()> {
        collection::create_collection(context, name, description, uri)
        
    }
    pub fn add_to_collection(
        context: Context<AddToCollectionAccountConstraints>
    ) -> Result<()> {
        collection::add_to_collection(context)
    }
}