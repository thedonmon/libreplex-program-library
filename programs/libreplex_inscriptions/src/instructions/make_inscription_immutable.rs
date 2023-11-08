use crate::{EncodingType, MediaType};
use crate::{Inscription, InscriptionSummary};
use anchor_lang::prelude::*;

use anchor_lang::system_program;


#[event]
pub struct InscriptionEventUpdate {
    pub id: Pubkey,
    pub data: InscriptionEventData,
}

// limited by the max size of a PDA account, i.e. 10K (each inscription pubkey is 32 bytes)

#[derive(Accounts)]
pub struct MakeInscriptionImmutable<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account()]
    pub authority: Signer<'info>,

    #[account(init_if_needed, seeds = [b"inscription_summary"],
        bump, payer = payer, space = InscriptionSummary::BASE_SIZE)]
    pub inscription_summary: Box<Account<'info, InscriptionSummary>>,

    /// CHECK: validated in logic
    #[account(mut,
        constraint = inscription.authority == authority.key())]
    pub inscription: Account<'info, Inscription>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<MakeInscriptionImmutable>) -> Result<()> {
    let inscription = &mut ctx.accounts.inscription;
    let inscription_summary = &mut ctx.accounts.inscription_summary;
    // let inscription_account_info = inscription.to_account_info();

    // we set the auth to the system program. This prevents any further changes
    inscription.authority = system_program::ID;

    inscription_summary.inscription_count_immutables += 1;
    let clock = Clock::get()?;

    inscription_summary.last_inscription_create_time = clock.unix_timestamp;
    inscription_summary.last_inscription = inscription.key();
    inscription_summary.last_inscriber = ctx.accounts.payer.key();

    emit!(InscriptionEventUpdate {
        id: inscription.key(),
        data: InscriptionEventData {
            authority: inscription.authority,
            root: inscription.root,
            media_type: inscription.media_type.clone(),
            encoding_type: inscription.encoding_type.clone(),
            inscription_data: inscription.inscription_data,
            order: inscription.order,
            size: inscription.size,
            validation_hash: inscription.validation_hash.clone()
        },
    });

    Ok(())
}