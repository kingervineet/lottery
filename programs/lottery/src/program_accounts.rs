use anchor_lang::prelude::*;

//lottery account
#[account]
#[derive(Default)]
pub struct Lottery {
    pub tickets_sold: u64,
    pub lottery_number: u32,
    pub lottery_value: u64,
    pub start_time: u64,
    pub end_time: u64,
}
impl Lottery {
    pub const MAX_SIZE: usize = 8 + 4 + 8 + 8 + 8;
}

//lottery account
#[account]
#[derive(Default)]
pub struct GlobalState {
    pub lotteries: u32,
    pub active_lottery: Option<Pubkey>,
    pub admin_authority: Pubkey,
    pub fee: u8,
    pub pause: bool,
}
impl GlobalState {
    pub const MAX_SIZE: usize = 4 + (1 + 32) + 32 + 1 + 1;
}
