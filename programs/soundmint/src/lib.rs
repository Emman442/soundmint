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
}