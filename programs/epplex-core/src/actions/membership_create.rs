use crate::*;

pub use anchor_lang::{
    prelude::*,
    solana_program::{
        program::{invoke, invoke_signed},
        sysvar::rent::ID as RENT_ID,
    },
};


#[derive(Accounts)]
pub struct MembershipCreate<'info> {
    #[account(mut)]
    pub membership: Signer<'info>,

    #[account(
        mut,
        seeds = [
            payer.key().as_ref(),
            token22_program.key().as_ref(),
            membership.key().as_ref()
        ],
        seeds::program = associated_token_program.key(),
        bump
    )]
    /// CHECK
    pub membership_ata: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = rule_creator.key() == rule.rule_creator
            @EphemeralityError::EscalatedAuthority,
    )]
    pub rule_creator: Signer<'info>,

    #[account(
        seeds = [
            SEED_EPHEMERAL_RULE,
            rule.seed.to_le_bytes().as_ref()
        ],
        bump = rule.bump,
    )]
    pub rule: Account<'info, EphemeralRule>,

    // Payer also becomes the owner
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = EphemeralData::INIT_SPACE,
        seeds = [
            SEED_EPHEMERAL_DATA,
            membership.key().as_ref()
        ],
        bump,
    )]
    pub data: Account<'info, EphemeralData>,

    #[account(
        // mut,
        seeds = [
            SEED_EPHEMERAL_AUTH
        ],
        bump
    )]
    /// CHECK:
    pub epplex_authority: UncheckedAccount<'info>,

    #[account(address = RENT_ID)]
    /// CHECK: this is fine since we are hard coding the rent sysvar.
    pub rent: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token22_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

impl<'info> MembershipCreate<'info> {
    pub fn create(
        &mut self,
        time: i64,
        name: String,
        symbol: String,
        uri: String,
        bumps: MembershipCreateBumps,
    ) -> Result<()> {
        // Step 0: Populate the EphemeralData account so we can reference it to use it.
        self.data.set_inner(EphemeralData {
            bump: bumps.data,
            mint: self.membership.key(),
            rule: self.rule.key(),
            expiry_time: Clock::get()?.unix_timestamp + time,
        });

        // Step 1: Initialize Account
        let size = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&[
            ExtensionType::MintCloseAuthority,
            ExtensionType::PermanentDelegate,
            ExtensionType::MetadataPointer,
            ExtensionType::TransferHook,
        ])
        .unwrap();

        let metadata = TokenMetadata {
            update_authority: anchor_spl::token_interface::spl_pod::optional_keys::OptionalNonZeroPubkey::try_from(Some(
                self.epplex_authority.key(),
            ))
            .unwrap(),
            mint: self.membership.key(),
            name,
            symbol,
            uri,
            additional_metadata: vec![],
        };

        let extension_extra_space = metadata.tlv_size_of().unwrap();
        let rent = &Rent::from_account_info(&self.rent.to_account_info())?;
        let lamports = rent.minimum_balance(size + extension_extra_space);

        invoke(
            &solana_program::system_instruction::create_account(
                &self.payer.key(),
                &self.membership.key(),
                lamports,
                (size).try_into().unwrap(),
                &spl_token_2022::id(),
            ),
            &[
                self.payer.to_account_info(),
                self.membership.to_account_info(),
            ],
        )?;

        // Step 2: Initialize Extension needed:

        // 2.1: Permanent Delegate,
        invoke(
            &initialize_permanent_delegate(
                &self.token22_program.key(),
                &self.membership.key(),
                &self.epplex_authority.key(),
            )?,
            &[self.membership.to_account_info()],
        )?;

        // 2.2: Transfer Hook,
        invoke(
            &intialize_transfer_hook(
                &self.token22_program.key(),
                &self.membership.key(),
                Some(self.epplex_authority.key()),
                None, // TO-DO: Add Transfer Hook
            )?,
            &[self.membership.to_account_info()],
        )?;

        // 2.3: Close Mint Authority,
        invoke(
            &initialize_mint_close_authority(
                &self.token22_program.key(),
                &self.membership.key(),
                Some(&self.epplex_authority.key()),
            )?,
            &[self.membership.to_account_info()],
        )?;

        // 2.4: Metadata Pointer
        invoke(
            &initialize_metadata_pointer(
                &self.token22_program.key(),
                &self.membership.key(),
                Some(self.epplex_authority.key()),
                Some(self.membership.key()),
            )?,
            &[self.membership.to_account_info()],
        )?;

        // Step 3: Initialize Mint & Metadata Account
        invoke(
            &initialize_mint2(
                &self.token22_program.key(),
                &self.membership.key(),
                &self.payer.key(),
                None,
                0,
            )?,
            &[self.membership.to_account_info()],
        )?;

        let seeds: &[&[u8]; 2] = &[SEED_EPHEMERAL_AUTH, &[bumps.epplex_authority]];
        let signer_seeds = &[&seeds[..]];

        invoke_signed(
            &initialize_metadata_account(
                &self.token22_program.key(),
                &self.membership.key(),
                &self.epplex_authority.key(),
                &self.membership.key(),
                &self.payer.key(),
                metadata.name,
                metadata.symbol,
                metadata.uri,
            ),
            &[
                self.membership.to_account_info(),
                self.epplex_authority.to_account_info(),
                self.payer.to_account_info(),
            ],
            signer_seeds,
        )?;

        // Step 4: Initialize the ATA & Mint to ATA + Changing Mint Authority to None so that nobody can mint anymore tokens.:

        // 4.1: Initialize ATA
        create(CpiContext::new(
            self.associated_token_program.to_account_info(),
            Create {
                payer: self.payer.to_account_info(), // payer
                associated_token: self.membership_ata.to_account_info(),
                authority: self.payer.to_account_info(), // owner
                mint: self.membership.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token22_program.to_account_info(),
            },
        ))?;

        // 4.2: Mint to ATA
        mint_to(
            CpiContext::new(
                self.token22_program.to_account_info(),
                MintTo {
                    mint: self.membership.to_account_info(),
                    to: self.membership_ata.to_account_info(),
                    authority: self.payer.to_account_info(),
                },
            ),
            1,
        )?;

        // 4.3: Removing mint authority
        set_authority(
            CpiContext::new(
                self.token22_program.to_account_info(),
                anchor_spl::token_interface::SetAuthority {
                    current_authority: self.payer.to_account_info().clone(),
                    account_or_mint: self.membership.to_account_info().clone(),
                },
            ),
            AuthorityType::MintTokens,
            None, // Set mint authority to be None
        )?;

        Ok(())
    }
}
