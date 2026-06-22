pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("2iGPHFEBSQ7sk5sFKmfSY9Y94wLAJHQguEH7kgTYBq4H");

#[program]
pub mod guild {
    use super::*;

    /// Initialize the protocol config (one-time).
    pub fn initialize(
        ctx: Context<Initialize>,
        stake_mint: Pubkey,
        min_bond: u64,
        fee_bps: u16,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, stake_mint, min_bond, fee_bps)
    }

    /// Register a new agent identity with a staked bond.
    pub fn register_agent(ctx: Context<RegisterAgent>, handle: String, bond: u64) -> Result<()> {
        instructions::register_agent::handler(ctx, handle, bond)
    }

    /// Add more collateral to an existing agent's bond.
    pub fn top_up_bond(ctx: Context<TopUpBond>, amount: u64) -> Result<()> {
        instructions::top_up_bond::handler(ctx, amount)
    }

    /// Post a job with an escrowed bounty.
    pub fn post_job(
        ctx: Context<PostJob>,
        bounty: u64,
        required_stake: u64,
        min_trust: TrustLevel,
        spec_uri: String,
    ) -> Result<()> {
        instructions::post_job::handler(ctx, bounty, required_stake, min_trust, spec_uri)
    }

    /// Claim an open job, locking the required stake.
    pub fn claim_job(ctx: Context<ClaimJob>) -> Result<()> {
        instructions::claim_job::handler(ctx)
    }

    /// Submit delivered work for review.
    pub fn submit_work(ctx: Context<SubmitWork>, delivery_uri: String) -> Result<()> {
        instructions::submit_work::handler(ctx, delivery_uri)
    }

    /// Review submitted work: accept (release escrow) or reject (slash stake).
    pub fn review_job(ctx: Context<ReviewJob>, approve: bool) -> Result<()> {
        instructions::review_job::handler(ctx, approve)
    }

    /// Cancel an unclaimed job and refund the bounty.
    pub fn cancel_job(ctx: Context<CancelJob>) -> Result<()> {
        instructions::cancel_job::handler(ctx)
    }
}
