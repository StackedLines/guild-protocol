use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::constants::*;
use crate::error::GuildError;
use crate::state::{Config, Job, JobStatus, TrustLevel};

#[derive(Accounts)]
pub struct PostJob<'info> {
    #[account(mut)]
    pub poster: Signer<'info>,

    #[account(mut, seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = poster,
        space = 8 + Job::INIT_SPACE,
        seeds = [JOB_SEED, config.job_count.to_le_bytes().as_ref()],
        bump
    )]
    pub job: Account<'info, Job>,

    /// CHECK: vault PDA escrowing the bounty for this job.
    #[account(mut, seeds = [VAULT_SEED, job.key().as_ref()], bump)]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<PostJob>,
    bounty: u64,
    required_stake: u64,
    min_trust: TrustLevel,
    spec_uri: String,
) -> Result<()> {
    require!(spec_uri.as_bytes().len() <= 200, GuildError::UriTooLong);

    // Escrow the bounty in the job vault.
    transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            Transfer {
                from: ctx.accounts.poster.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        bounty,
    )?;

    let clock = Clock::get()?;
    let config = &mut ctx.accounts.config;
    let id = config.job_count;

    let job = &mut ctx.accounts.job;
    job.id = id;
    job.poster = ctx.accounts.poster.key();
    job.worker = Pubkey::default();
    job.reviewer = ctx.accounts.poster.key();
    job.bounty = bounty;
    job.required_stake = required_stake;
    job.min_trust = min_trust;
    job.spec_uri = spec_uri;
    job.delivery_uri = String::new();
    job.status = JobStatus::Open;
    job.created_at = clock.unix_timestamp;
    job.updated_at = clock.unix_timestamp;
    job.bump = ctx.bumps.job;

    config.job_count = config.job_count.checked_add(1).ok_or(GuildError::Overflow)?;

    msg!("Job {} posted: bounty={} stake={}", id, bounty, required_stake);
    Ok(())
}
