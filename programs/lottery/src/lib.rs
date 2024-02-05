use anchor_lang::prelude::*;

//use anchor_lang::solana_program::stake::instruction;
use anchor_lang::solana_program::{program::invoke, system_instruction};
use std::borrow::Borrow;

mod constants;
mod errors;
mod program_accounts;

use constants::*;
use errors::*;
use program_accounts::*;

declare_id!("Dkt4r29UNxFWqTZCe1bHWFmn97s1KYLiabr8NNU8x3Mb");

#[program]
pub mod lottery {

    use anchor_lang::{
        accounts,
        solana_program::clock,
        system_program::{transfer, Transfer},
    };

    use super::*;

    // Initialize the contract
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let accounts: &mut Initialize<'_> = ctx.accounts;

        // Initialize the Global State Account
        accounts.global_state_account.lotteries = 0;
        accounts.global_state_account.active_lottery = None;
        accounts.global_state_account.admin_authority = accounts.signer.key();
        accounts.global_state_account.pause = false;
        accounts.global_state_account.fee = 0;

        msg!("Program initialized!");
        Ok(())
    }

    pub fn change_config(
        ctx: Context<ChangeConfig>,
        new_fee: Option<u8>,
        new_admin: Option<Pubkey>,
    ) -> Result<()> {
        let accounts: &mut ChangeConfig<'_> = ctx.accounts;

        // Only admin can run
        require!(
            accounts.signer.key() == accounts.global_state_account.admin_authority,
            LotteryError::NotAuthorized
        );

        // Check if contract is paused
        require!(
            !accounts.global_state_account.pause,
            LotteryError::ContractIsPaused
        );

        accounts.global_state_account.fee = new_fee.unwrap_or(accounts.global_state_account.fee);
        accounts.global_state_account.admin_authority =
            new_admin.unwrap_or(accounts.global_state_account.admin_authority);

        msg!("config changed!");
        Ok(())
    }

    // Change Config method is run by admin and is used to change the different fees and admin account
    pub fn create_lottery(
        ctx: Context<CreateLottery>,
        lottery_number: String,
        lotery_value: u64,
        duration: u64,
    ) -> Result<()> {
        let accounts: &mut CreateLottery<'_> = ctx.accounts;
        let clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp as u64;

        // Only admin can run
        require!(
            accounts.admin.key() == accounts.global_state_account.admin_authority,
            LotteryError::NotAuthorized
        );

        // Check if contract is paused
        require!(
            !accounts.global_state_account.pause,
            LotteryError::ContractIsPaused
        );

        require!(
            accounts.global_state_account.active_lottery.is_none(),
            LotteryError::AnotherLotteryActive
        );

        accounts.global_state_account.lotteries += 1;
        accounts.global_state_account.active_lottery = Some(accounts.lottery_account.key());

        accounts.lottery_account.start_time = timestamp;
        accounts.lottery_account.end_time = timestamp + duration;
        accounts.lottery_account.lottery_value = lotery_value;
        accounts.lottery_account.tickets_sold = 0;
        accounts.lottery_account.lottery_number = accounts.global_state_account.lotteries;

        msg!("Lottery Account Created!");
        Ok(())
    }

    // This method is run by admin and creates the mint.
    pub fn buy_tickets(ctx: Context<BuyTickets>, tickets: u64, bump: u8) -> Result<()> {
        let accounts: &mut BuyTickets<'_> = ctx.accounts;
        let clock: Clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp as u64;

        // Check if contract is paused
        require!(
            !accounts.global_state_account.pause,
            LotteryError::ContractIsPaused
        );

        // Check if lottery already ended
        require!(
            timestamp < accounts.lottery_account.end_time,
            LotteryError::LotteryAlreadyEnded
        );

        // Verify admin account
        require!(
            accounts.admin.key() == accounts.global_state_account.admin_authority,
            LotteryError::WrongAdminAccount
        );

        accounts.lottery_account.tickets_sold += tickets;

        let amount = tickets * accounts.lottery_account.lottery_value;

        // Calculate the fee
        let fee_to_be_paid = (accounts.global_state_account.fee as u64 * amount) / 100;

        transfer(
            CpiContext::new(
                accounts.system_program.to_account_info(),
                Transfer {
                    from: accounts.signer.to_account_info(),
                    to: accounts.lottery_account.to_account_info(),
                },
            ),
            amount,
        )?;

        transfer(
            CpiContext::new(
                accounts.system_program.to_account_info(),
                Transfer {
                    from: accounts.signer.to_account_info(),
                    to: accounts.admin.to_account_info(),
                },
            ),
            fee_to_be_paid,
        )?;

        Ok(())
    }

    pub fn winner_payout(ctx: Context<WinnerPayout>) -> Result<()> {
        let accounts: &mut WinnerPayout<'_> = ctx.accounts;

        // Check if contract is paused
        require!(
            !accounts.global_state_account.pause,
            LotteryError::ContractIsPaused
        );

        // Verify admin account
        require!(
            accounts.admin.key() == accounts.global_state_account.admin_authority,
            LotteryError::NotAuthorized
        );

        // Get the minimum amount required for rent
        let rent_due =
            Rent::get()?.minimum_balance(accounts.lottery_account.to_account_info().data_len());

        // Balance is Total balance of account - Rent
        let balance = accounts.lottery_account.to_account_info().lamports() - rent_due;

        **accounts
            .lottery_account
            .to_account_info()
            .try_borrow_mut_lamports()? -= balance;
        **accounts
            .winner
            .to_account_info()
            .try_borrow_mut_lamports()? += balance;

        accounts.global_state_account.active_lottery = None;

        accounts
            .lottery_account
            .close(accounts.admin.to_account_info())?;

        Ok(())
    }

    // Run by admin to pause all features of the contract
    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        let accounts = ctx.accounts;

        // Only admin can run
        require!(
            accounts.admin.key() == accounts.global_state_account.admin_authority,
            LotteryError::NotAuthorized
        );

        // Check if contract is paused
        require!(
            !accounts.global_state_account.pause,
            LotteryError::ContractIsPaused
        );

        accounts.global_state_account.pause = true;
        msg!("Contract Paused!");
        Ok(())
    }

    // Run by admin to resume all features of the contract
    pub fn resume(ctx: Context<Resume>) -> Result<()> {
        let accounts = ctx.accounts;

        // Only admin can run
        require!(
            accounts.admin.key() == accounts.global_state_account.admin_authority,
            LotteryError::NotAuthorized
        );

        accounts.global_state_account.pause = false;
        msg!("Contract Resumed!");
        Ok(())
    }
}

//initialization Accounts
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        space = 8 + GlobalState::MAX_SIZE,
        payer = signer,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state_account: Account<'info, GlobalState>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

//initialization Accounts
#[derive(Accounts)]
pub struct ChangeConfig<'info> {
    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state_account: Account<'info, GlobalState>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

//change config instruction accounts
#[derive(Accounts)]
#[instruction(lottery_number: String)]

pub struct CreateLottery<'info> {
    #[account(
        init,
        space = 8 + Lottery::MAX_SIZE,
        payer = admin,
        seeds = [LOTTERY_SEED, lottery_number.as_bytes()],
        bump,
    )]
    pub lottery_account: Account<'info, Lottery>,
    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state_account: Account<'info, GlobalState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

//create wrapped token accounts
#[derive(Accounts)]
pub struct BuyTickets<'info> {
    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state_account: Account<'info, GlobalState>,
    #[account(
        mut,
        seeds = [LOTTERY_SEED, global_state_account.lotteries.to_string().as_bytes()],
        bump,
    )]
    pub lottery_account: Account<'info, Lottery>,
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK: safe
    #[account(mut)]
    pub admin: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

//creating/update a user account and adding a pending claim entry instruction accounts
#[derive(Accounts)]
pub struct WinnerPayout<'info> {
    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state_account: Account<'info, GlobalState>,
    #[account(
        mut,
        seeds = [LOTTERY_SEED, global_state_account.lotteries.to_string().as_bytes()],
        bump,
    )]
    pub lottery_account: Account<'info, Lottery>,
    /// CHECK: safe
    #[account(mut)]
    pub winner: AccountInfo<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

//Pause instruction accounts
#[derive(Accounts)]
pub struct Pause<'info> {
    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state_account: Account<'info, GlobalState>,
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

//Pause instruction accounts
#[derive(Accounts)]
pub struct Resume<'info> {
    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state_account: Account<'info, GlobalState>,
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}
