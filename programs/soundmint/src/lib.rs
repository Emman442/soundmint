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
}