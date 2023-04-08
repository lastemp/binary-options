use anchor_lang::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Insufficient amount to withdraw.")]
    InvalidWithdrawAmount,
    #[msg("Amount must be greater than zero.")]
    AmountNotgreaterThanZero,
    #[msg("Withdrawal amount exceeds total payout amount.")]
    ExceededTotalPayoutAmount,
    #[msg("Withdrawal amount must be equal to total payout amount.")]
    AmountNotEqualToTotalPayoutAmount,
    #[msg("Deposit amount must be equal to bet_amount.")]
    InvalidDepositAmount,
    #[msg("Participant must make a prediction and win it.")]
    InvalidPrediction,
    #[msg("Winning Amount exceeds deposited amount.")]
    InvalidWinningAmount,
    #[msg("Betting cannot be created, missing data")]
    CannotCreateBetting,
    #[msg("Exceeded betting description max length")]
    ExceededDescriptionMaxLength,
    #[msg("Both predictions cannot not be same.")]
    PredictionCannotBeSame,
    #[msg("Single participant is not allowed to take both predictions.")]
    PredictionDisAllowed,
    #[msg("Participant not allowed to make a withdrawal.")]
    WithdrawalDisAllowed,
    #[msg("Invalid participant winner.")]
    InvalidWinner,
    #[msg("Create options not initialised or participants limit of two cannot be exceeded.")]
    InvalidParticipantsLimit,
    #[msg("Account is not initialized.")]
    AccountNotInitialized,
    #[msg("Account is already initialized.")]
    AccountAlreadyInitialized,
    //
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
    #[msg("The config has already been initialized.")]
    ReInitialize,
    #[msg("The config has not been initialized.")]
    UnInitialize,
    #[msg("Argument is invalid.")]
    InvalidArgument,
    #[msg("An overflow occurs.")]
    Overflow,
    #[msg("Pyth has an internal error.")]
    PythError,
    #[msg("Pyth price oracle is offline.")]
    PythOffline,
    #[msg("The loan value is higher than the collateral value.")]
    LoanValueTooHigh,
    #[msg("Program should not try to serialize a price account.")]
    TryToSerializePriceAccount,
}