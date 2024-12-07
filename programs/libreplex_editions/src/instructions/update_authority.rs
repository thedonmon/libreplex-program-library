use spl_token_metadata_interface::{state::TokenMetadata, instruction::update_authority};
use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use anchor_spl::token_2022::{self, spl_token_2022::{extension::{BaseStateWithExtensions, StateWithExtensions}, state::Mint}};
use spl_pod::optional_keys::OptionalNonZeroPubkey; 

use crate::{EditionsDeployment, errors::EditionsError};


#[derive(Accounts)]
pub struct UpdateMintAuthorityCtx<'info> {

    #[account(mut,
        seeds = ["editions_deployment".as_ref(), editions_deployment.symbol.as_ref()], bump)]
    pub editions_deployment: Account<'info, EditionsDeployment>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub mint: AccountInfo<'info>,
    
    /* BOILERPLATE PROGRAM ACCOUNTS */
    /// CHECK: Checked in constraint
    #[account(
        constraint = token_program.key() == token_2022::ID
    )]
    pub token_program: UncheckedAccount<'info>,

    #[account()]
    pub system_program: Program<'info, System>,

}

pub fn update_mint_authority<'info>(ctx: Context<'_, '_, '_, 'info, UpdateMintAuthorityCtx<'info>>) -> Result<()> {
    let mint = &ctx.accounts.mint;
    let token_program = &ctx.accounts.token_program;
    let editions_deployment = &ctx.accounts.editions_deployment;
    let payer = &ctx.accounts.payer;

    require!(editions_deployment.max_number_of_tokens == editions_deployment.number_of_tokens_issued, EditionsError::MintNotComplete);
    require!(editions_deployment.creator.key() == payer.key(), EditionsError::InvalidCreator);
    
    let mint_data = mint.data.borrow();
    let mint_with_extension = StateWithExtensions::<Mint>::unpack(&mint_data)?;
    let metadata = mint_with_extension
    .get_variable_len_extension::<TokenMetadata>()
    .ok()
    .map(|x| x.clone());

    require!(metadata.is_some(), EditionsError::MetadataNotFound);
    let expected_update_authority = OptionalNonZeroPubkey::try_from(Some(editions_deployment.key()))?;

    if metadata.unwrap().update_authority.ne(&expected_update_authority) {
        return Err(EditionsError::UpdateAuthorityAlreadyChanged.into());
    }

    let deployment_seeds: &[&[u8]] = &[
            "editions_deployment".as_bytes(),
            editions_deployment.symbol.as_ref(),
            &[ctx.bumps.editions_deployment],
        ];

    let account_infos = [
        payer.to_account_info(),
        editions_deployment.to_account_info(),
        mint.to_account_info(),
        token_program.to_account_info(),
    ];

    let payer = OptionalNonZeroPubkey::try_from(Some(payer.to_account_info().key()))?;

    let update_authority_ix = update_authority(
        &spl_token_2022::ID,
        &mint.key(),
        &editions_deployment.key(),
        payer
    );

    invoke_signed(&update_authority_ix, &account_infos, &[deployment_seeds])?;

    Ok(())
}