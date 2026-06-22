use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::GuildError;
use crate::state::{AgentProfile, Config, Job, JobStatus, TrustLevel};

#[derive(Accounts)]
pub struct ReviewJob<'info> {
    #[account(mut)]
    pub reviewer: Signer<'info>,

    #[account(seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [JOB_SEED, job.id.to_le_bytes().as_ref()],
        bump = job.bump,
        constraint = job.reviewer == reviewer.key() @ GuildError::Unauthorized
    )]
    pub job: Account<'info, Job>,

    /// CHECK: job vault holding the escrowed bounty (PDA, system-owned).
    #[account(mut, seeds = [VAULT_SEED, job.key().as_ref()], bump)]
    pub job_vault: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [AGENT_SEED, worker.owner.as_ref()],
        bump = worker.bump,
        constraint = worker.key() == job.worker @ GuildError::Unauthorized
    )]
    pub worker: Account<'info, AgentProfile>,

    /// CHECK: worker's bond vault (PDA, system-owned).
    #[account(mut, seeds = [VAULT_SEED, worker.key().as_ref()], bump)]
    pub worker_vault: UncheckedAccount<'info>,

    /// CHECK: wallet that receives the worker payout (worker owner).
    #[account(mut, address = worker.owner)]
    pub worker_wallet: UncheckedAccount<'info>,

    /// CHECK: poster wallet, receives refund on rejection.
    #[account(mut, address = job.poster)]
    pub poster_wallet: UncheckedAccount<'info>,

    /// CHECK: protocol treasury, receives fees and slashed stake.
    #[account(mut, address = config.treasury)]
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

/// Move lamports out of a system-owned PDA by direct balance mutation.
fn pay_from_vault(vault: &AccountInfo, to: &AccountInfo, amount: u64) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    **vault.try_borrow_mut_lamports()? = vault
        .lamports()
        .checked_sub(amount)
        .ok_or(GuildError::Overflow)?;
    **to.try_borrow_mut_lamports()? = to
        .lamports()
        .checked_add(amount)
        .ok_or(GuildError::Overflow)?;
    Ok(())
}

fn promote(reputation: u64) -> TrustLevel {
    if reputation >= TRUST_SENIOR_THRESHOLD {
        TrustLevel::Senior
    } else if reputation >= TRUST_TRUSTED_THRESHOLD {
        TrustLevel::Trusted
    } else if reputation >= TRUST_STANDARD_THRESHOLD {
        TrustLevel::Standard
    } else {
        TrustLevel::Probation
    }
}

pub fn handler(ctx: Context<ReviewJob>, approve: bool) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(job.status == JobStatus::Submitted, GuildError::InvalidJobStatus);

    let worker = &mut ctx.accounts.worker;
    let bounty = job.bounty;
    let stake = job.required_stake;

    // Unlock the worker's at-risk stake regardless of outcome.
    worker.locked_bond = worker
        .locked_bond
        .checked_sub(stake)
        .ok_or(GuildError::Overflow)?;

    if approve {
        // Protocol fee on the bounty.
        let fee = (bounty as u128)
            .checked_mul(ctx.accounts.config.fee_bps as u128)
            .ok_or(GuildError::Overflow)?
            .checked_div(10_000)
            .ok_or(GuildError::Overflow)? as u64;
        let payout = bounty.checked_sub(fee).ok_or(GuildError::Overflow)?;

        pay_from_vault(
            &ctx.accounts.job_vault.to_account_info(),
            &ctx.accounts.worker_wallet.to_account_info(),
            payout,
        )?;
        pay_from_vault(
            &ctx.accounts.job_vault.to_account_info(),
            &ctx.accounts.treasury.to_account_info(),
            fee,
        )?;

        worker.jobs_completed = worker.jobs_completed.checked_add(1).ok_or(GuildError::Overflow)?;
        worker.reputation = worker
            .reputation
            .checked_add(REPUTATION_PER_JOB)
            .ok_or(GuildError::Overflow)?;
        worker.trust = promote(worker.reputation);

        job.status = JobStatus::Accepted;
        msg!("Job {} accepted. payout={} fee={}", job.id, payout, fee);
    } else {
        // Slash the worker's stake to the treasury.
        pay_from_vault(
            &ctx.accounts.worker_vault.to_account_info(),
            &ctx.accounts.treasury.to_account_info(),
            stake,
        )?;
        worker.bond = worker.bond.checked_sub(stake).ok_or(GuildError::Overflow)?;
        worker.jobs_slashed = worker.jobs_slashed.checked_add(1).ok_or(GuildError::Overflow)?;
        worker.reputation = worker.reputation.saturating_sub(REPUTATION_PER_JOB);
        worker.trust = promote(worker.reputation);

        // Refund the escrowed bounty to the poster.
        pay_from_vault(
            &ctx.accounts.job_vault.to_account_info(),
            &ctx.accounts.poster_wallet.to_account_info(),
            bounty,
        )?;

        job.status = JobStatus::Rejected;
        msg!("Job {} rejected. slashed={}", job.id, stake);
    }

    job.updated_at = Clock::get()?.unix_timestamp;
    Ok(())
}
