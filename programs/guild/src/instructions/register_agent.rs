use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::constants::*;
use crate::error::GuildError;
use crate::state::{AgentProfile, Config, TrustLevel};

#[derive(Accounts)]
pub struct RegisterAgent<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = owner,
        space = 8 + AgentProfile::INIT_SPACE,
        seeds = [AGENT_SEED, owner.key().as_ref()],
        bump
    )]
    pub agent: Account<'info, AgentProfile>,

    /// Vault PDA that custodies the agent's bonded lamports.
    /// CHECK: system-owned PDA used purely to hold lamports.
    #[account(
        mut,
        seeds = [VAULT_SEED, agent.key().as_ref()],
        bump
    )]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RegisterAgent>, handle: String, bond: u64) -> Result<()> {
    require!(handle.as_bytes().len() <= 32, GuildError::HandleTooLong);
    require!(bond >= ctx.accounts.config.min_bond, GuildError::BondTooLow);

    // Move the bond from the owner into the vault PDA.
    transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            Transfer {
                from: ctx.accounts.owner.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        bond,
    )?;

    let agent = &mut ctx.accounts.agent;
    agent.owner = ctx.accounts.owner.key();
    agent.handle = handle;
    agent.bond = bond;
    agent.locked_bond = 0;
    agent.jobs_completed = 0;
    agent.jobs_slashed = 0;
    agent.reputation = 0;
    agent.trust = TrustLevel::Probation;
    agent.active = true;
    agent.bump = ctx.bumps.agent;

    let config = &mut ctx.accounts.config;
    config.agent_count = config.agent_count.checked_add(1).ok_or(GuildError::Overflow)?;

    msg!("Agent registered: {} bond={}", agent.handle, bond);
    Ok(())
}
