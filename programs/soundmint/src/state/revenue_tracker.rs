use anchor_lang::prelude::*;

#[account]
pub struct RevenueTracker {
    pub master_nft: Pubkey,
    pub total_revenue: u64,
    pub streaming_revenue: u64, 
    pub sales_revenue: u64,
    pub other_revenue: u64,
    pub transactions: Vec<RevenueTransaction>,
    pub created_at: i64,
    pub last_revenue_timestamp: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RevenueTransaction {
    pub amount: u64,
    pub source: String,
    pub description: String,
    pub timestamp: i64,
}

impl RevenueTracker {
    pub const DISCRIMINATOR_LENGTH: usize = 8;
    pub const PUBKEY_LENGTH: usize = 32;
    pub const U64_LENGTH: usize = 8;
    pub const I64_LENGTH: usize = 8;
    pub const U8_LENGTH: usize = 1;
    pub const STRING_PREFIX_LENGTH: usize = 4;
    pub const VECTOR_PREFIX_LENGTH: usize = 4;
    
    // Assuming a maximum of 100 transaction records
    pub const MAX_TRANSACTIONS: usize = 100;
    pub const TRANSACTION_SIZE: usize = 
        Self::U64_LENGTH +                      // amount
        Self::STRING_PREFIX_LENGTH + 20 +       // source (estimating 20 bytes)
        Self::STRING_PREFIX_LENGTH + 100 +      // description (estimating 100 bytes)
        Self::I64_LENGTH;                       // timestamp
    
    pub const INIT_SPACE: usize = 
        Self::DISCRIMINATOR_LENGTH +
        Self::PUBKEY_LENGTH +                              // master_nft
        Self::U64_LENGTH +                                 // total_revenue
        Self::U64_LENGTH +                                 // streaming_revenue
        Self::U64_LENGTH +                                 // sales_revenue
        Self::U64_LENGTH +                                 // other_revenue
        Self::VECTOR_PREFIX_LENGTH +                       // transactions vector prefix
        Self::MAX_TRANSACTIONS * Self::TRANSACTION_SIZE +  // transactions
        Self::I64_LENGTH +                                 // created_at
        Self::I64_LENGTH +                                 // last_revenue_timestamp
        Self::U8_LENGTH;                                   // bump
}