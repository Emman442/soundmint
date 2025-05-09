use anchor_lang::prelude::*;

#[account]
pub struct MasterNft {
    pub title: String,
    pub description: String,
    pub artist_profile: Pubkey,
    pub audio_uri: String,
    pub artwork_uri: String,
    pub metadata: Vec<MetadataItem>,
    pub mint: Pubkey,
    pub is_transferable: bool,
    pub status: MasterNftStatus,
    pub created_at: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum MasterNftStatus {
    Active,
    Delisted,
    Frozen,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MetadataItem {
    pub key: String,
    pub value: String,
}

impl MasterNft {
    pub const DISCRIMINATOR_LENGTH: usize = 8;
    pub const PUBKEY_LENGTH: usize = 32;
    pub const BOOL_LENGTH: usize = 1;
    pub const U8_LENGTH: usize = 1;
    pub const I64_LENGTH: usize = 8;
    pub const STRING_PREFIX_LENGTH: usize = 4;
    pub const VECTOR_PREFIX_LENGTH: usize = 4;
    pub const ENUM_LENGTH: usize = 1; // Simple enum variant discriminator
    
    pub const INIT_SPACE: usize = 
        Self::DISCRIMINATOR_LENGTH +
        Self::STRING_PREFIX_LENGTH + 100 +      // title
        Self::STRING_PREFIX_LENGTH + 500 +      // description
        Self::PUBKEY_LENGTH +                   // artist_profile
        Self::STRING_PREFIX_LENGTH + 200 +      // audio_uri
        Self::STRING_PREFIX_LENGTH + 200 +      // artwork_uri
        Self::VECTOR_PREFIX_LENGTH +            // metadata vector prefix
        10 * (Self::STRING_PREFIX_LENGTH + 50 + // key (max 10 items)
              Self::STRING_PREFIX_LENGTH + 50) +// value (max 10 items)
        Self::PUBKEY_LENGTH +                   // mint
        Self::BOOL_LENGTH +                     // is_transferable
        Self::ENUM_LENGTH +                     // status
        Self::I64_LENGTH +                      // created_at
        Self::U8_LENGTH;                        // bump
}