    use crate::state::{Metadata};
use crate::{ CreateMetadataInput, MetadataEvent, MetadataEventType, assert_pda_derivation::assert_pda_derivation};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::{errors::ErrorCode};



// whitelisted signer programs

pub mod migrator_lite {
    use super::*;
    declare_id!("migr1m1An7f3X75nKuuUn9mm3844miK62ZohpqRfQHp");
}


#[derive(Accounts)]
#[instruction(metadata_input: CreateMetadataInput)]
pub struct CreateMetadata<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(init, seeds = [b"metadata", mint.key().as_ref()],
              bump, payer = payer, space = Metadata::BASE_SIZE + metadata_input.get_size())]
    pub metadata: Box<Account<'info, Metadata>>,

    pub mint: Box<Account<'info, Mint>>,

    /*
        Authority needs to be a mint or a PDA generated by a whitelisted migrator program
    */
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,


    /*
     only to be supplied if the migration is invoked by a whitelisted 
     migrator program.

     if a migrator program is invoked, then the signer account must be
     a PDA derived by the migrator program from seed [mint].
    */
    pub invoked_migrator_program: Option<UncheckedAccount<'info>> 
}

pub fn handler(ctx: Context<CreateMetadata>, metadata_input: CreateMetadataInput) -> Result<()> {
    let metadata = &mut ctx.accounts.metadata;
    let mint = &mut ctx.accounts.mint;
    let authority = &ctx.accounts.authority;
    let invoked_migrator_program = &ctx.accounts.invoked_migrator_program;


    assert_is_valid_signer(&authority.key(), &mint.key(), invoked_migrator_program)?;

    // Update the metadata state account
    metadata.mint = ctx.accounts.mint.key();
    metadata.is_mutable = true;
    metadata.symbol = metadata_input.symbol.clone();
    metadata.name = metadata_input.name.clone();
    metadata.creator = authority.key();
    metadata.description = metadata_input.description;
    metadata.asset = metadata_input.asset;
    metadata.update_authority = metadata_input.update_authority;

    msg!(
        "metadata created for mint with pubkey {}",
        ctx.accounts.mint.key()
    );

    emit!(MetadataEvent {
        id: metadata.key(),
        mint: ctx.accounts.mint.key(),
        event_type: MetadataEventType::Create
    });

    Ok(())
}

fn assert_is_valid_signer<'info> (signer: &Pubkey, mint: &Pubkey, invoked_migrator_program: &Option<UncheckedAccount<'info>>) -> Result<()> {
    match invoked_migrator_program {
        Some(x) => {

            // currently migrator is the only whitelisted signer program 
            if x.key() != migrator_lite::ID  {
                return err!(ErrorCode::InvalidSignedProgram);
            }

            let seeds = [
                b"metadata_signer",
                mint.as_ref()
            ];

            msg!("{} {}", x.key(), signer.key());
            assert_pda_derivation(&x.key(), signer, &seeds)?;

        },
        None => {
            // no migrator invoked. Hence mint must be the signer

        }
    }

    return Ok(())
}