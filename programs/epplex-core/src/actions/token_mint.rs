use crate::*;
use epplex_metadata::program::EpplexMetadata;
use anchor_spl::token_interface::MintTo;
use epplex_shared::Token2022;

#[derive(Accounts)]
#[instruction(params: TokenCreateParams)]
pub struct TokenMint<'info> {
    #[account(mut, signer)]
    /// CHECK
    pub mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK
    pub ata: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK
    pub token_metadata: UncheckedAccount<'info>,

    // TODO: is unchecked account correct?
    #[account()]
    /// CHECK
    pub permanent_delegate: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token22_program: Program<'info, Token2022>,
    pub associated_token: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, EpplexMetadata>
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct TokenCreateParams {
    pub destroy_timestamp_offset: i64,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

impl TokenMint<'_> {

    pub fn validate(&self, _ctx: &Context<Self>, _params: &TokenCreateParams) -> Result<()> {
        Ok(())
    }

    // This function should be a general purpose minter
    pub fn actuate(ctx: Context<Self>, _params: TokenCreateParams) -> Result<()> {
        // let update_authority =
        //     OptionalNonZeroPubkey::try_from(Some(deployment.key())).expect("Bad update auth");

        // Create the ephemeral token
        init_mint_account(
            ctx.accounts.payer.to_account_info().clone(),
            ctx.accounts.mint.to_account_info().clone(),
            ctx.accounts.rent.to_account_info().clone(),
            &[
                ExtensionType::PermanentDelegate,
                ExtensionType::MintCloseAuthority,
                ExtensionType::MetadataPointer
            ]
        )?;

        // TODO Need to create a separate PDA that simply has a bump - can do this later
        // Could also simply check for the permanent delegate address

        // Create metadata account
        // create_metadata_account(
        //     ctx.accounts.metadata_program.to_account_info().clone(),
        //     ctx.accounts.payer.to_account_info().clone(),
        //     ctx.accounts.mint.to_account_info().clone(),
        //     ctx.accounts.token_metadata.to_account_info().clone(),
        //     ctx.accounts.system_program.to_account_info().clone(),
        //     params
        // )?;

        // Add metadata pointer
        add_metadata_pointer(
            ctx.accounts.token22_program.key(),
            &ctx.accounts.mint.to_account_info(),
            // TODO: who should have authority here - permanent delegate should be passed in
            ctx.accounts.permanent_delegate.key(),
            ctx.accounts.mint.key(),
        )?;

        // Initialize the actual mint data
        initialize_mint(
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.rent.to_account_info(),
            &ctx.accounts.token22_program.key(),
            // TODO incorrect mint auth
            &ctx.accounts.payer.key(),
            // TODO incorrect freeze auth
            &ctx.accounts.payer.key(),
        )?;

        // Create ATA
        anchor_spl::associated_token::create(
            CpiContext::new(
                ctx.accounts.token22_program.to_account_info(),
                anchor_spl::associated_token::Create {
                    payer: ctx.accounts.payer.to_account_info(), // payer
                    associated_token: ctx.accounts.ata.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(), // owner
                    mint: ctx.accounts.mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token22_program.to_account_info(),
                }
            ),
        )?;

        // Mint to ATA
        anchor_spl::token_interface::mint_to(
            CpiContext::new(
                ctx.accounts.token22_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info().clone(),
                    to: ctx.accounts.ata.to_account_info().clone(),
                    authority: ctx.accounts.payer.to_account_info(),
                }
            ),
            1
        )?;

        // TODO after minting should prolly burn the mint auth

        Ok(())
    }

}