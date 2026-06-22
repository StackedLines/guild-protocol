use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::GuildError;
use crate::state::{AgentProfile, Job, JobStatus};

fn trust_rank(t: &crate::state::TrustLevel) -> u8 {
    use crate::state::TrustLevel::*;
    match t {
        Probation => 0,
        Standard => 1,
        Trusted => 2,
        Senior => 3,
    }
}

#[derive(Accounts)]
pub struct ClaimJob<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [AGENT_SEED, owner.key().as_ref()],
        bump = agent.bump,
        has_one = owner @ GuildError::Unauthorized
    )]
    pub agent: Account<'info, AgentProfile>,

    #[account(mut, seeds = [JOB_SEED, job.id.to_le_bytes().as_ref()], bump = job.bump)]
    pub job: Account<'info, Job>,
}

pub fn handler(ctx: Context<ClaimJob>) -> Result<()> {
    let agent = &mut ctx.accounts.agent;
    let job = &mut ctx.accounts.job;

    require!(agent.active, GuildError::AgentInactive);
    require!(job.status == JobStatus::Open, GuildError::InvalidJobStatus);
    require!(
        trust_rank(&agent.trust) >= trust_rank(&job.min_trust),
        GuildError::TrustTooLow
    );

    let free = agent.bond.checked_sub(agent.locked_bond).ok_or(GuildError::Overflow)?;
    require!(free >= job.required_stake, GuildError::InsufficientFreeBond);

    agent.locked_bond = agent
        .locked_bond
        .checked_add(job.required_stake)
        .ok_or(GuildError::Overflow)?;

    job.worker = agent.key();
    job.status = JobStatus::Claimed;
    job.updated_at = Clock::get()?.unix_timestamp;

    msg!("Job {} claimed by {}", job.id, agent.handle);
    Ok(())
}
