use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized access to this account")]
    Unauthorized,
    
    #[msg("String is too long")]
    StringTooLong,
    
    #[msg("Too many social links")]
    TooManySocialLinks,
    
    #[msg("Invalid amount provided")]
    InvalidAmount,

    #[msg("Insufficient funds for this operation")]
    InsufficientFunds,
    
    #[msg("Invalid royalty split configuration")]
    InvalidRoyaltySplit,
    
    #[msg("Royalty shares must add up to 100%")]
    InvalidRoyaltyShares,
    
    #[msg("No revenue available to claim")]
    NoRevenueToClaim,
    
    #[msg("Invalid NFT metadata")]
    InvalidNftMetadata,
    
    #[msg("Artist profile not found")]
    ArtistProfileNotFound,
    
    #[msg("Master NFT not found")]
    MasterNftNotFound,

    #[msg("Too many collaborators")]
    TooManyCollaborators,

    #[msg("Too many metadata items")]
    TooManyMetadataItems,

    #[msg("Amount too small to process")]
    AmountTooSmall,

    #[msg("Invalid fee percentage")]
    InvalidFeePercentage,

    #[msg("Invalid treasury wallet")]
    InvalidTreasuryWallet,
}