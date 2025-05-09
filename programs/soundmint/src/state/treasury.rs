use anchor_lang::prelude::*;

#[account]
pub struct Treasury {
    pub authority: Pubkey,
    pub treasury_wallet: Pubkey,
    pub mint_fee: u64,
    pub platform_fee_basis_points: u16,
    pub total_revenue_collected: u64,
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

impl Treasury {
    pub const DISCRIMINATOR_LENGTH: usize = 8;
    pub const PUBKEY_LENGTH: usize = 32;
    pub const U64_LENGTH: usize = 8;
    pub const U16_LENGTH: usize = 2;
    pub const I64_LENGTH: usize = 8;
    pub const U8_LENGTH: usize = 1;
    
    pub const INIT_SPACE: usize = 
        Self::DISCRIMINATOR_LENGTH +
        Self::PUBKEY_LENGTH +     // authority
        Self::PUBKEY_LENGTH +     // treasury_wallet
        Self::U64_LENGTH +        // mint_fee
        Self::U16_LENGTH +        // platform_fee_basis_points
        Self::U64_LENGTH +        // total_revenue_collected
        Self::I64_LENGTH +        // created_at
        Self::I64_LENGTH +        // updated_at
        Self::U8_LENGTH;          // bump
}