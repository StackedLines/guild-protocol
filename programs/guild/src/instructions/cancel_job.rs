use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke_signed, system_instruction};

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

    let job_key = job.key();
    let vault_bump = ctx.bumps.job_vault;
    let refund = job.bounty;
    if refund > 0 {
        let seeds: &[&[u8]] = &[VAULT_SEED, job_key.as_ref(), &[vault_bump]];
        invoke_signed(
            &system_instruction::transfer(
                ctx.accounts.job_vault.key,
                ctx.accounts.poster.key,
                refund,
            ),
            &[
                ctx.accounts.job_vault.to_account_info(),
                ctx.accounts.poster.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[seeds],
        )?;
    }

    job.status = JobStatus::Cancelled;
    job.updated_at = Clock::get()?.unix_timestamp;
    msg!("Job {} cancelled, {} refunded", job.id, refund);
    Ok(())
}
