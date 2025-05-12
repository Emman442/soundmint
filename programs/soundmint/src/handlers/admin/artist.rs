use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::CustomError;
use crate::constants::*;

pub fn verify_artist(
    context: Context<VerifyArtistAccountConstraints>,
    verify: bool
) -> Result<()> {
    let artist_profile = &mut context.accounts.artist_profile;
    
    // Only treasury authority can verify artists
    require!(
        context.accounts.treasury.authority == context.accounts.authority.key(),
        CustomError::Unauthorized
    );
    
    artist_profile.is_verified = verify;
    
    if verify {
        msg!("Artist {} has been verified", artist_profile.name);
    } else {
        msg!("Artist {} verification has been removed", artist_profile.name);
    }
    
    Ok(())
}

#[derive(Accounts)]
pub struct VerifyArtistAccountConstraints<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = artist_profile.authority == artist_owner.key() @ CustomError::Unauthorized
    )]
    pub artist_profile: Account<'info, ArtistProfile>,
    
    /// CHECK: This is the artist's wallet
    pub artist_owner: UncheckedAccount<'info>,
    
    #[account(
        seeds = [TREASURY_SEED],
        bump = treasury.bump,
        constraint = treasury.authority == authority.key() @ CustomError::Unauthorized
    )]
    pub treasury: Account<'info, Treasury>,
    
    pub system_program: Program<'info, System>,
}