use anchor_lang::prelude::*;

#[error_code]
pub enum LotteryError {
    #[msg("Lottery has already Ended.")]
    LotteryAlreadyEnded,
    #[msg("Not Authorized to perform this operation")]
    NotAuthorized,
    #[msg("Contract is Paused")]
    ContractIsPaused,
    #[msg("The Admin Account being sent is not correct")]
    WrongAdminAccount,
    #[msg("There is another lottery active !!")]
    AnotherLotteryActive,
}
