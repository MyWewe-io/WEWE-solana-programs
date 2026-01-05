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

    #[msg("Pool validation failed: insufficient funds transferred")]
    InsufficientFundsTransferred,

    #[msg("Pool validation failed: pool not properly initialized")]
    PoolNotInitialized,

    #[msg("Pool validation failed: invalid pool state")]
    InvalidPoolState,

    #[msg("Pool validation failed: position NFT not created")]
    PositionNFTNotCreated,

    #[msg("Pool validation failed: token vault account mismatch")]
    InvalidTokenVault,

    #[msg("Pool validation failed: WSOL vault account mismatch")]
    InvalidWSolVault,

    #[msg("Emergency unlock: pool actually exists")]
    PoolActuallyExists,

    #[msg("Emergency unlock: too soon for emergency unlock")]
    TooSoonForEmergencyUnlock,

    #[msg("Emergency unlock: unauthorized")]
    UnauthorizedEmergencyUnlock,

    #[msg("Maximum number of backed proposals reached")]
    MaxBackedProposalsReached,

    #[msg("Invalid price range: sqrt_price must be between sqrt_min_price and sqrt_max_price")]
    InvalidPriceRange,

    #[msg("Invalid parameters: liquidity must be greater than 0")]
    InvalidParameters,

    #[msg("Liquidity cannot be zero. Ensure base_amount and quote_amount are non-zero and sqrt_price is within valid range.")]
    LiquidityCannotBeZero,

    #[msg("Insufficient funds in vault to cover proposal requirements")]
    InsufficientFunds,
}
