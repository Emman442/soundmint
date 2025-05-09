use anchor_lang::prelude::*;
use crate::state::*;
use crate::CustomError;

pub fn create_artist_profile(context: Context<CreateArtistProfileAccountConstraints>, name: String, description: String, profile_image_uri: String) -> Result<()> {
    require!(name.len() <= ArtistProfile::MAX_NAME_LENGTH, CustomError::StringTooLong);
    require!(description.len() <= ArtistProfile::MAX_DESCRIPTION_LENGTH, CustomError::StringTooLong);
    require!(profile_image_uri.len() <= ArtistProfile::MAX_URI_LENGTH, CustomError::StringTooLong);
    
    let artist_profile = &mut context.accounts.artist_profile;
    let clock = Clock::get()?;
    
    artist_profile.authority = context.accounts.authority.key();
    artist_profile.name = name;
    artist_profile.description = description;
    artist_profile.profile_image_uri = profile_image_uri;
    artist_profile.social_links = Vec::new();
    artist_profile.is_verified = false;
    artist_profile.track_count = 0;
    artist_profile.created_at = clock.unix_timestamp;
    artist_profile.updated_at = clock.unix_timestamp;
    artist_profile.bump = context.bumps.artist_profile;
    
    msg!("Artist profile created for: {}", artist_profile.name);
    Ok(())
}

pub fn update_artist_profile(
    context: Context<UpdateArtistProfileAccountConstraints>, 
    name: Option<String>, 
    description: Option<String>, 
    profile_image_uri: Option<String>,
    social_links: Option<Vec<SocialLink>>
) -> Result<()> {
    let artist_profile = &mut context.accounts.artist_profile;
    let clock = Clock::get()?;
    
    if let Some(new_name) = name {
        require!(new_name.len() <= ArtistProfile::MAX_NAME_LENGTH, CustomError::StringTooLong);
        artist_profile.name = new_name;
    }
    
    if let Some(new_description) = description {
        require!(new_description.len() <= ArtistProfile::MAX_DESCRIPTION_LENGTH, CustomError::StringTooLong);
        artist_profile.description = new_description;
    }
    
    if let Some(new_uri) = profile_image_uri {
        require!(new_uri.len() <= ArtistProfile::MAX_URI_LENGTH, CustomError::StringTooLong);
        artist_profile.profile_image_uri = new_uri;
    }
    
    if let Some(new_social_links) = social_links {
        require!(new_social_links.len() <= ArtistProfile::MAX_SOCIAL_LINKS, CustomError::TooManySocialLinks);
        
        // Validate each social link
        for link in &new_social_links {
            require!(link.platform.len() <= ArtistProfile::MAX_PLATFORM_LENGTH, CustomError::StringTooLong);
            require!(link.url.len() <= ArtistProfile::MAX_URL_LENGTH, CustomError::StringTooLong);
        }
        
        artist_profile.social_links = new_social_links;
    }
    
    artist_profile.updated_at = clock.unix_timestamp;
    
    msg!("Artist profile updated for: {}", artist_profile.name);
    Ok(())
}

#[derive(Accounts)]
pub struct CreateArtistProfileAccountConstraints<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = ArtistProfile::INIT_SPACE,
        seeds = [b"artist_profile", authority.key().as_ref()],
        bump
    )]
    pub artist_profile: Account<'info, ArtistProfile>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateArtistProfileAccountConstraints<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"artist_profile", authority.key().as_ref()],
        bump = artist_profile.bump,
        constraint = artist_profile.authority == authority.key() @ CustomError::Unauthorized
    )]
    pub artist_profile: Account<'info, ArtistProfile>,
    
    pub system_program: Program<'info, System>,
}