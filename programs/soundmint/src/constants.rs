use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";

// Basis points conversion (100% = 10,000 basis points)
pub const TOTAL_BASIS_POINTS: u16 = 10_000;

// Default fees
pub const DEFAULT_MINT_FEE: u64 = 10_000_000;  // 0.01 SOL in lamports
pub const DEFAULT_PLATFORM_FEE: u16 = 500;     // 5% in basis points

// NFT config
pub const MAX_METADATA_ITEMS: usize = 10;

// Collection configuration
pub const SOUND_MINT_COLLECTION_PREFIX: &[u8] = b"sound_mint_collection";

// Treasury configuration
pub const TREASURY_SEED: &[u8] = b"treasury";

// PDA seeds
pub const ARTIST_PROFILE_SEED: &[u8] = b"artist_profile";
pub const MASTER_NFT_SEED: &[u8] = b"master_nft";
pub const ROYALTY_SPLIT_SEED: &[u8] = b"royalty_split";
pub const ROYALTY_NFT_SEED: &[u8] = b"royalty_nft";
pub const REVENUE_TRACKER_SEED: &[u8] = b"revenue_tracker";