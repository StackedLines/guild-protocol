use anchor_lang::prelude::*;

/// Global protocol configuration. One per deployment (PDA: ["config"]).
#[account]
#[derive(InitSpace)]
pub struct Config {
    /// Authority allowed to update protocol parameters.
    pub authority: Pubkey,
    /// Mint used for staking bonds and job escrow (e.g. SOL wrapper or SPL token).
    pub stake_mint: Pubkey,
    /// Minimum bond an agent must lock to register, in base units.
    pub min_bond: u64,
    /// Protocol fee in basis points (e.g. 250 = 2.5%) taken from settled jobs.
    pub fee_bps: u16,
    /// Destination that accrues protocol fees (the open-source treasury).
    pub treasury: Pubkey,
    /// Total number of agents registered (monotonic counter).
    pub agent_count: u64,
    /// Total number of jobs created (monotonic counter, also used as job id).
    pub job_count: u64,
    /// Bump for the config PDA.
    pub bump: u8,
}

/// Trust tier an agent has earned. Higher tiers unlock larger jobs.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum TrustLevel {
    Probation,
    Standard,
    Trusted,
    Senior,
}

/// On-chain identity + reputation for an autonomous agent.
/// PDA: ["agent", owner].
#[account]
#[derive(InitSpace)]
pub struct AgentProfile {
    /// Wallet that controls this agent.
    pub owner: Pubkey,
    /// Short handle, max 32 bytes.
    #[max_len(32)]
    pub handle: String,
    /// Bonded collateral currently locked for accountability.
    pub bond: u64,
    /// Portion of the bond currently locked against active jobs.
    pub locked_bond: u64,
    /// Count of jobs delivered and accepted.
    pub jobs_completed: u64,
    /// Count of jobs where the agent was slashed.
    pub jobs_slashed: u64,
    /// Accumulated reputation score (non-transferable, on-chain).
    pub reputation: u64,
    /// Current trust tier.
    pub trust: TrustLevel,
    /// Whether the agent is active and can take work.
    pub active: bool,
    /// Bump for the agent PDA.
    pub bump: u8,
}

/// Lifecycle of a job in the marketplace.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum JobStatus {
    /// Posted with escrowed bounty, open for an agent to claim.
    Open,
    /// Claimed by an agent, work in progress.
    Claimed,
    /// Work submitted, awaiting review.
    Submitted,
    /// Reviewed and accepted; escrow released to the worker.
    Accepted,
    /// Reviewed and rejected; worker stake slashed.
    Rejected,
    /// Cancelled by the poster before being claimed; escrow refunded.
    Cancelled,
}

/// A unit of paid work. PDA: ["job", job_id_le_bytes].
#[account]
#[derive(InitSpace)]
pub struct Job {
    /// Sequential id from Config.job_count.
    pub id: u64,
    /// Who posted and funded the job.
    pub poster: Pubkey,
    /// Agent profile that claimed the job (default until claimed).
    pub worker: Pubkey,
    /// Designated reviewer (the poster may self-review or assign another).
    pub reviewer: Pubkey,
    /// Escrowed bounty in stake_mint base units.
    pub bounty: u64,
    /// Stake the worker must put at risk to claim (slashed on rejection).
    pub required_stake: u64,
    /// Minimum trust tier required to claim.
    pub min_trust: TrustLevel,
    /// IPFS/arweave CID or URL describing the task, max 200 bytes.
    #[max_len(200)]
    pub spec_uri: String,
    /// CID/URL of the delivered work, set on submit, max 200 bytes.
    #[max_len(200)]
    pub delivery_uri: String,
    /// Current status.
    pub status: JobStatus,
    /// Unix timestamp when the job was created.
    pub created_at: i64,
    /// Unix timestamp of last status change.
    pub updated_at: i64,
    /// Bump for the job PDA.
    pub bump: u8,
}
