use anchor_lang::prelude::*;

#[account]
pub struct RoyaltyNft {
    pub master_nft: Pubkey,
    pub mint: Pubkey,
    pub share_basis_points: u16,
    pub amount_claimed: u64,
    pub last_claimed_at: i64,
    pub created_at: i64,
    pub bump: u8,
}

impl RoyaltyNft {
    pub const DISCRIMINATOR_LENGTH: usize = 8;
    pub const PUBKEY_LENGTH: usize = 32;
    pub const U16_LENGTH: usize = 2;
    pub const U64_LENGTH: usize = 8;
    pub const I64_LENGTH: usize = 8;
    pub const U8_LENGTH: usize = 1;
    
    pub const INIT_SPACE: usize = 
        Self::DISCRIMINATOR_LENGTH +
        Self::PUBKEY_LENGTH +     // master_nft
        Self::PUBKEY_LENGTH +     // mint
        Self::U16_LENGTH +        // share_basis_points
        Self::U64_LENGTH +        // amount_claimed
        Self::I64_LENGTH +        // last_claimed_at
        Self::I64_LENGTH +        // created_at
        Self::U8_LENGTH;          // bump
}