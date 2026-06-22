use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::constants::*;
use crate::error::GuildError;
use crate::state::AgentProfile;

#[derive(Accounts)]
pub struct TopUpBond<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [AGENT_SEED, owner.key().as_ref()],
        bump = agent.bump,
        has_one = owner @ GuildError::Unauthorized
    )]
    pub agent: Account<'info, AgentProfile>,

    /// CHECK: vault PDA holding the agent's lamports.
    #[account(mut, seeds = [VAULT_SEED, agent.key().as_ref()], bump)]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<TopUpBond>, amount: u64) -> Result<()> {
    transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            Transfer {
                from: ctx.accounts.owner.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        amount,
    )?;

    let agent = &mut ctx.accounts.agent;
    agent.bond = agent.bond.checked_add(amount).ok_or(GuildError::Overflow)?;
    msg!("Bond topped up by {} -> {}", amount, agent.bond);
    Ok(())
}
