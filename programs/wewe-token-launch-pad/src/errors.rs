use anchor_lang::error_code;

#[error_code]
pub enum ProposalError {
    #[msg("Minimum funding goal has not been reached yet.")]
    TargetNotMet,

    #[msg("Maximum funding goal has already been reached.")]
    BackingGoalReached,

    #[msg("You have already backed this proposal.")]
    ProposalAlreadyBacked,

    #[msg("Backing period is still active. Please wait until it ends.")]
    BackingNotEnded,

    #[msg("Backing period has ended. You can no longer contribute.")]
    BackingEnded,

    #[msg("Only the proposal owner is authorized to perform this action.")]
    NotOwner,

    #[msg("This proposal has been rejected. You may claim your refund.")]
    ProposalRejected,
}

