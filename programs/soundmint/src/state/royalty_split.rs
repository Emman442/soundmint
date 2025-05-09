use anchor_lang::prelude::*;

#[account]
pub struct RoyaltySplit {
    pub master_nft: Pubkey,
    pub collaborators: Vec<Collaborator>,
    pub total_basis_points: u16,
    pub total_revenue_collected: u64,
    pub created_at: i64,
    pub last_revenue_timestamp: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Collaborator {
    pub address: Pubkey,
    pub name: String,
    pub share_basis_points: u16,
    pub amount_claimed: u64,
}

impl RoyaltySplit {
    pub const DISCRIMINATOR_LENGTH: usize = 8;
    pub const PUBKEY_LENGTH: usize = 32;
    pub const U16_LENGTH: usize = 2;
    pub const U64_LENGTH: usize = 8;
    pub const I64_LENGTH: usize = 8;
    pub const U8_LENGTH: usize = 1;
    pub const STRING_PREFIX_LENGTH: usize = 4;
    pub const VECTOR_PREFIX_LENGTH: usize = 4;
    
    // Assuming a maximum of 10 collaborators for space calculation
    pub const MAX_COLLABORATORS: usize = 10;
    pub const COLLABORATOR_SIZE: usize = 
        Self::PUBKEY_LENGTH +                // address
        Self::STRING_PREFIX_LENGTH + 50 +    // name (estimating 50 bytes)
        Self::U16_LENGTH +                   // share_basis_points
        Self::U64_LENGTH;                    // amount_claimed
    
    pub const INIT_SPACE: usize = 
        Self::DISCRIMINATOR_LENGTH +
        Self::PUBKEY_LENGTH +                        // master_nft
        Self::VECTOR_PREFIX_LENGTH +                 // collaborators vector prefix
        Self::MAX_COLLABORATORS * Self::COLLABORATOR_SIZE + // collaborators
        Self::U16_LENGTH +                           // total_basis_points
        Self::U64_LENGTH +                           // total_revenue_collected
        Self::I64_LENGTH +                           // created_at
        Self::I64_LENGTH +                           // last_revenue_timestamp
        Self::U8_LENGTH;                             // bump
}