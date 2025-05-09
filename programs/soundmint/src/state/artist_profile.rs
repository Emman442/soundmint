use anchor_lang::prelude::*;

#[account]
pub struct ArtistProfile {
    pub authority: Pubkey,
    pub name: String,
    pub description: String,
    pub profile_image_uri: String,
    pub social_links: Vec<SocialLink>,
    pub is_verified: bool,
    pub track_count: u64,
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SocialLink {
    pub platform: String,
    pub url: String,
}

impl ArtistProfile {
    pub const DISCRIMINATOR_LENGTH: usize = 8;
    pub const PUBKEY_LENGTH: usize = 32;
    pub const BOOL_LENGTH: usize = 1;
    pub const U64_LENGTH: usize = 8;
    pub const I64_LENGTH: usize = 8;
    pub const U8_LENGTH: usize = 1;
    pub const STRING_PREFIX_LENGTH: usize = 4; // Length prefix for strings
    pub const VECTOR_PREFIX_LENGTH: usize = 4; // Length prefix for vectors

    pub const MAX_NAME_LENGTH: usize = 50;
    pub const MAX_DESCRIPTION_LENGTH: usize = 200;
    pub const MAX_URI_LENGTH: usize = 200;
    pub const MAX_SOCIAL_LINKS: usize = 5;
    pub const MAX_PLATFORM_LENGTH: usize = 20;
    pub const MAX_URL_LENGTH: usize = 100;
    
    pub const INIT_SPACE: usize = 
        Self::DISCRIMINATOR_LENGTH +
        Self::PUBKEY_LENGTH +                   // authority
        Self::STRING_PREFIX_LENGTH + 50 +       // name
        Self::STRING_PREFIX_LENGTH + 200 +      // description
        Self::STRING_PREFIX_LENGTH + 200 +      // profile_image_uri
        Self::VECTOR_PREFIX_LENGTH +            // social_links vector prefix
        5 * (Self::STRING_PREFIX_LENGTH + 20 +  // platform (max 5 items)
             Self::STRING_PREFIX_LENGTH + 100) + // url (max 5 items)
        Self::BOOL_LENGTH +                     // is_verified
        Self::U64_LENGTH +                      // track_count
        Self::I64_LENGTH +                      // created_at
        Self::I64_LENGTH +                      // updated_at
        Self::U8_LENGTH;                        // bump
}