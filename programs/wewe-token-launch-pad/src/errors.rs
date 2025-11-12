use anchor_lang::error_code;

#[error_code]
pub enum ProposalError {
    #[msg("Numerical overflow occurred.")]
    NumericalOverflow,
    
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

    #[msg("This proposal is already rejected")]
    ProposalAlreadyRejected,

    #[msg("Pool for this proposal is already launched.")]
    PoolAlreadyLaunched,

    #[msg("Create a wallet through wewe app to contribute")]
    NotAuthorised,

    #[msg("You cannot back your own proposal")]
    CantBackOwnProposal,

    #[msg("Amount can't be higher than the allowed amount per user per airdrop")]
    AmountTooBig,

    #[msg("Amount already updated for the current airdrop cycle of the proposal.")]
    AmountAlreadyUpdated,

    #[msg("Type conversion failed")]
    TypeCastFailed,

    #[msg("Lenth of token metadat is too long")]
    LenthTooLong,

    #[msg("Account passed is Incorrect")]
    IncorrectAccount,

    #[msg("Initial Airdrop received already")]
    AirdropAlreadyRecived,

    #[msg("No milestone is active")]
    NoMilestoneActive,

    #[msg("score of all backer not updated for current milestone")]
    AllBackerScoreNotUpdated,

    #[msg("Maximum number of backed proposals reached")]
    MaxBackedProposalsReached,
}
