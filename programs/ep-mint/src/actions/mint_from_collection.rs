use crate::*;

use ephemerality::{TokenCreateFromCollectionParams, ONE_WEEK};


#[derive(Accounts)]
pub struct MintFromCollection<'info> {
    #[account(mut)]
    pub minter: Signer<'info>,

    #[account(
        mut,
        seeds = [
            GUARD_SEED,
            collection_config.key().as_ref()
        ],
        bump = mint_guard.bump
    )]
    pub mint_guard: Account<'info, MintGuard>,

    pub epplex_program: Program<'info, Ephemerality>,

    // TODO: need to place constraint on collection_name
    #[account(
        seeds = [
            SEED_COLLECTION_CONFIG,
            &mint_guard.collection_counter.to_be_bytes(),
        ],
        seeds::program = ephemerality::ID.key(),
        bump = collection_config.bump
    )]
    pub collection_config: Account<'info, CollectionConfig>,

    #[account(mut, signer)]
    /// CHECK
    pub token_mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK
    pub ata: UncheckedAccount<'info>,

    // #[account(
    //     seeds = [
    //         SEED_TOKEN_METADATA,
    //         ephemerality::ID.key().as_ref(),
    //         token_mint.key().as_ref()
    //     ],
    //     seeds::program = ephemerality::ID.key(),
    //     bump,
    // )]
    // pub token_metadata: Account<'info, EphemeralMetadata>,

    #[account(mut)]
    /// CHECK
    pub token_metadata: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK
    pub program_delegate: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub token22_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub associated_token: Program<'info, AssociatedToken>
}

impl MintFromCollection<'_> {
    
    pub fn validate(&self, ctx: &Context<Self>) -> Result<()> {

        let mint_guard = &ctx.accounts.mint_guard;
        let collection_config = &ctx.accounts.collection_config;

        if collection_config.collection_size <= mint_guard.items_minted {
            return err!(MintError::CollectionMintedOut);
        };

        Ok(())
    }

    pub fn actuate(ctx: Context<Self>) -> Result<()> {
        let collection_config = &mut ctx.accounts.collection_config;
        let mint_guard = &mut ctx.accounts.mint_guard;

        //create cpi

        let cpi_accounts = ephemerality::cpi::accounts::TokenCreateFromCollection {
            mint: ctx.accounts.token_mint.to_account_info().clone(),
            ata: ctx.accounts.ata.to_account_info().clone(),
            token_metadata: ctx.accounts.token_metadata.to_account_info().clone(),
            program_delegate: ctx.accounts.program_delegate.to_account_info().clone(),
            payer: ctx.accounts.minter.to_account_info().clone(),
            rent: ctx.accounts.rent.to_account_info().clone(),
            token22_program: ctx.accounts.token22_program.to_account_info().clone(),
            system_program: ctx.accounts.system_program.to_account_info().clone(),
            associated_token: ctx.accounts.associated_token.to_account_info().clone(),
            collection_config: collection_config.to_account_info().clone(),
            mint_authority: mint_guard.to_account_info().clone(),
            treasury: mint_guard.to_account_info().clone()
        };

        // let cpi_ctx = CpiContext::new_with_signer(
        //     cpi_program,
        //     cpi_accounts
        // );

        // let seeds = &[SEED_LOTTO, lotto_id.as_ref(), &[lotto.bump]];
        // token::transfer(
        //     CpiContext::new_with_signer(
        //         token_program.to_account_info(),
        //         Transfer {
        //             from: token_vault.to_account_info(),
        //             to: token_owner_account.to_account_info(),
        //             authority: lotto.to_account_info(),
        //         },
        //         &[&seeds[..]],
        //     ),
        //     amount,
        // )

        //create token creation params
        let mut token_name = collection_config.collection_name.to_owned();
        token_name.push_str(&mint_guard.items_minted.to_string());

        let params = TokenCreateFromCollectionParams {
            destroy_timestamp_offset: ONE_WEEK,
            name: token_name,
            //TODO add collection symbol to collection config
            symbol: collection_config.collection_symbol.to_owned(),
            uri: "".to_string()
        };


        let collection_config_key = collection_config.key();

        let seeds = &[GUARD_SEED, collection_config_key.as_ref(), &[mint_guard.bump]];
        ephemerality::cpi::token_create_from_collection(
            CpiContext::new_with_signer(
                ctx.accounts.epplex_program.to_account_info(),
                cpi_accounts,
                &[seeds]
            ),
            params
        )?;

        mint_guard.items_minted += 1;

        Ok(())
    }

}
