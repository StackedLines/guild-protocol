use anchor_lang::prelude::*;

#[constant]
pub const CONFIG_SEED: &[u8] = b"config";

#[constant]
pub const AGENT_SEED: &[u8] = b"agent";

#[constant]
pub const JOB_SEED: &[u8] = b"job";

#[constant]
pub const VAULT_SEED: &[u8] = b"vault";

/// Reputation awarded for a job accepted on first review.
pub const REPUTATION_PER_JOB: u64 = 10;

/// Reputation thresholds for trust-tier promotion.
pub const TRUST_STANDARD_THRESHOLD: u64 = 30;
pub const TRUST_TRUSTED_THRESHOLD: u64 = 100;
pub const TRUST_SENIOR_THRESHOLD: u64 = 300;

/// Maximum protocol fee (10%) to bound governance.
pub const MAX_FEE_BPS: u16 = 1000;
