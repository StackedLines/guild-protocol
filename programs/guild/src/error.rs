use anchor_lang::prelude::*;

#[error_code]
pub enum GuildError {
    #[msg("Handle exceeds maximum length of 32 bytes")]
    HandleTooLong,
    #[msg("Spec or delivery URI exceeds maximum length")]
    UriTooLong,
    #[msg("Bond is below the protocol minimum")]
    BondTooLow,
    #[msg("Fee exceeds the maximum allowed basis points")]
    FeeTooHigh,
    #[msg("Agent does not have enough free (unlocked) bond")]
    InsufficientFreeBond,
    #[msg("Agent is not active")]
    AgentInactive,
    #[msg("Agent trust level is below the job requirement")]
    TrustTooLow,
    #[msg("Job is not in the required status for this action")]
    InvalidJobStatus,
    #[msg("Caller is not authorized for this action")]
    Unauthorized,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Cannot withdraw while bond is locked against active jobs")]
    BondLocked,
}
