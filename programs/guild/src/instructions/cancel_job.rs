use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::GuildError;
use crate::state::{Job, JobStatus};

#[derive(Accounts)]
pub struct CancelJob<'info> {
    #[account(mut, address = job.poster)]
    pub poster: Signer<'info>,

    #[account(
        mut,
        seeds = [JOB_SEED, job.id.to_le_bytes().as_ref()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,

    /// CHECK: job vault holding the escrowed bounty (PDA, system-owned).
    #[account(mut, seeds = [VAULT_SEED, job.key().as_ref()], bump)]
    pub job_vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CancelJob>) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(job.status == JobStatus::Open, GuildError::InvalidJobStatus);

    let vault = ctx.accounts.job_vault.to_account_info();
    let poster = ctx.accounts.poster.to_account_info();
    let refund = job.bounty;
    if refund > 0 {
        **vault.try_borrow_mut_lamports()? = vault
            .lamports()
            .checked_sub(refund)
            .ok_or(GuildError::Overflow)?;
        **poster.try_borrow_mut_lamports()? = poster
            .lamports()
            .checked_add(refund)
            .ok_or(GuildError::Overflow)?;
    }

    job.status = JobStatus::Cancelled;
    job.updated_at = Clock::get()?.unix_timestamp;
    msg!("Job {} cancelled, {} refunded", job.id, refund);
    Ok(())
}
