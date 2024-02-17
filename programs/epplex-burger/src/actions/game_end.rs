use anchor_lang::prelude::*;

use crate::{GameConfig, GamePhase, SEED_GAME_CONFIG};

#[derive(Accounts)]
pub struct GameEnd<'info> {
    #[account(address = game_config.game_master)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [SEED_GAME_CONFIG],
        bump = game_config.bump,
    )]
    pub game_config: Account<'info, GameConfig>,

    pub system_program: Program<'info, System>,
}

impl GameEnd<'_> {
    pub fn validate(&self, _ctx: &Context<Self>) -> Result<()> {
        self.game_config.check_game_ended()
    }

    pub fn actuate(ctx: Context<Self>) -> Result<()> {
        let game_config = &mut ctx.accounts.game_config;

        game_config.game_phase = GamePhase::Elimination;

        Ok(())
    }
}
