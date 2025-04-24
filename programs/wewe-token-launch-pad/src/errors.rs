use anchor_lang::error_code;

#[error_code]
pub enum ProposalError {
    #[msg("The amount to raise has not been met")]
    TargetNotMet,
    #[msg("The amount to raise has been achieved")]
    TargetMet,
    #[msg("The contribution is too big")]
    ContributionTooBig,
    #[msg("The contribution is too small")]
    ContributionTooSmall,
    #[msg("The maximum amount to contribute has been reached")]
    MaximumContributionsReached,
    #[msg("The backing has not ended yet")]
    BackingNotEnded,
    #[msg("The backing has ended")]
    BackingEnded,
    #[msg("Invalid total amount. i should be bigger than 3")]
    InvalidAmount,
    #[msg("Only owner can call this function!")]
    NotOwner,
    #[msg("This proposal is rejected")]
    ProposalRejected,
}