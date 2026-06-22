use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::GuildError;
use crate::state::{AgentProfile, Job, JobStatus};

#[derive(Accounts)]
pub struct SubmitWork<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [AGENT_SEED, owner.key().as_ref()],
        bump = agent.bump,
        has_one = owner @ GuildError::Unauthorized
    )]
    pub agent: Account<'info, AgentProfile>,

    #[account(
        mut,
        seeds = [JOB_SEED, job.id.to_le_bytes().as_ref()],
        bump = job.bump,
        constraint = job.worker == agent.key() @ GuildError::Unauthorized
    )]
    pub job: Account<'info, Job>,
}

pub fn handler(ctx: Context<SubmitWork>, delivery_uri: String) -> Result<()> {
    require!(delivery_uri.as_bytes().len() <= 200, GuildError::UriTooLong);

    let job = &mut ctx.accounts.job;
    require!(job.status == JobStatus::Claimed, GuildError::InvalidJobStatus);

    job.delivery_uri = delivery_uri;
    job.status = JobStatus::Submitted;
    job.updated_at = Clock::get()?.unix_timestamp;

    msg!("Job {} submitted for review", job.id);
    Ok(())
}
