use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::GuildError;
use crate::state::Config;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + Config::INIT_SPACE,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, Config>,

    /// CHECK: treasury is just a destination pubkey for accrued fees.
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Initialize>,
    stake_mint: Pubkey,
    min_bond: u64,
    fee_bps: u16,
) -> Result<()> {
    require!(fee_bps <= MAX_FEE_BPS, GuildError::FeeTooHigh);

    let config = &mut ctx.accounts.config;
    config.authority = ctx.accounts.authority.key();
    config.stake_mint = stake_mint;
    config.min_bond = min_bond;
    config.fee_bps = fee_bps;
    config.treasury = ctx.accounts.treasury.key();
    config.agent_count = 0;
    config.job_count = 0;
    config.bump = ctx.bumps.config;

    msg!("Guild initialized: min_bond={}, fee_bps={}", min_bond, fee_bps);
    Ok(())
}
