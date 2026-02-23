use anchor_lang::prelude::error_code;

#[error_code]
pub enum ContinuousTokenError {
    #[msg("Bad configuration parameters")]
    BadConfig,
    #[msg("Overflow")]
    Overflow,
    #[msg("Underflow")]
    Underflow,
    #[msg("Insufficient Balance.")]
    InsufficientBalance,
    #[msg("Invalid Amount.")]
    InvalidAmount,
    #[msg("Invalid Referral.")]
    InvalidReferral,
    #[msg("Invalid Referrer ATA.")]
    InvalidReferrerAta,
    #[msg("Self Referral not allowed.")]
    SelfReferralNotAllowed,
    #[msg("Incorrect Mint.")]
    IncorrectMint,
}
