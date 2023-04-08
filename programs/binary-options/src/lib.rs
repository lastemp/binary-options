use anchor_lang::{prelude::*, system_program};
//
use std::mem::size_of;
//use anchor_lang::prelude::*;
//use solana_program::account_info::AccountInfo;

pub mod state;
use state::PriceFeed;
use state::AdminConfig;

mod error;
use error::Errors;

declare_id!("JCqZPL84bJQKfQ1FZ4cYSWddYNpQCBXPjowjkCcdn9ZB");

const DESCRIPTION_LENGTH: usize = 40; // betting description length
const STALENESS_THRESHOLD : u64 = 1800; // staleness threshold in seconds 60

#[program]
pub mod binary_options {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, config: AdminConfig) -> Result<()> {
        let deposit_account = &mut ctx.accounts.admin_deposit_account;
        let config_account = &mut ctx.accounts.config;

        config_account.set_inner(config);

        deposit_account.admin_auth = *ctx.accounts.admin_auth.key;
        deposit_account.admin_auth_bump = *ctx.bumps.get("admin_pda_auth").unwrap();
        deposit_account.admin_sol_vault_bump = ctx.bumps.get("admin_sol_vault").copied();
        deposit_account.is_initialized = true;

        Ok(())
    }

    pub fn create_binary_options(ctx: Context<CreateBinaryOptions>, bet_description: String, bet_amount: u64, strike_price: u64, taker_amount: u64, participantPosition: ParticipantPosition) -> Result<()> {
        if bet_description.trim().is_empty() {
            return Err(Errors::CannotCreateBetting.into());
        }
        if bet_description.as_bytes().len() > DESCRIPTION_LENGTH {
            return Err(Errors::ExceededDescriptionMaxLength.into());
        }

        // bet_amount
        let valid_amount = {
            if bet_amount > 0 {
                true
            }
            else{false}
        };
        // amount must be greater than zero
        if !valid_amount {
            return Err(Errors::AmountNotgreaterThanZero.into());
        }
        
        // strike_price
        let valid_amount = {
            if strike_price > 0 {
                true
            }
            else{false}
        };
        // amount must be greater than zero
        if !valid_amount {
            return Err(Errors::AmountNotgreaterThanZero.into());
        }

        // taker_amount
        let valid_amount = {
            if taker_amount > 0 {
                true
            }
            else{false}
        };
        // amount must be greater than zero
        if !valid_amount {
            return Err(Errors::AmountNotgreaterThanZero.into());
        }
        
        let deposit_account = &mut ctx.accounts.deposit_account;
        let deposit_auth = &ctx.accounts.deposit_auth;
        let sys_program = &ctx.accounts.system_program;

        deposit_account.deposit_auth = *ctx.accounts.deposit_auth.key;
        deposit_account.taker_auth = *ctx.accounts.deposit_auth.key;
        deposit_account.auth_bump = *ctx.bumps.get("pda_auth").unwrap();
        deposit_account.sol_vault_bump = ctx.bumps.get("sol_vault").copied();
        deposit_account.bet_description = bet_description;
        deposit_account.bet_amount = bet_amount;
        deposit_account.strike_price = strike_price;
        deposit_account.taker_amount = taker_amount;
        deposit_account.first_participant = participantPosition;
        deposit_account.betting_state = 1;

        let cpi_accounts = system_program::Transfer {
            from: deposit_auth.to_account_info(),
            to: ctx.accounts.sol_vault.to_account_info(),
        };

        let cpi = CpiContext::new(sys_program.to_account_info(), cpi_accounts);

        system_program::transfer(cpi, bet_amount)?;

        Ok(())
    }

    //  accept binary options and deposit native sol
    pub fn accept_binary_options(ctx: Context<AcceptBinaryOptions>, amount: u64, participant_position: ParticipantPosition) -> Result<()> {
        let valid_amount = {
            if amount > 0 {
                true
            }
            else{false}
        };
        // amount must be greater than zero
        if !valid_amount {
            return Err(Errors::AmountNotgreaterThanZero.into());
        }
        
        let deposit_account = &mut ctx.accounts.deposit_account;
        let deposit_auth = &ctx.accounts.deposit_auth;
        let sys_program = &ctx.accounts.system_program;

        let valid_amount = {
            if amount == deposit_account.taker_amount {
                true
            }
            else{false}
        };
        // amount must be equal to taker_amount
        if !valid_amount {
            return Err(Errors::InvalidDepositAmount.into());
        }

        // first participant is not allowed to make prediction since they had previously done so in create options.
        if deposit_account.deposit_auth.eq(deposit_auth.key) {
            return Err(Errors::PredictionDisAllowed.into());
        }

        let first_participant_position = {
            match deposit_account.first_participant {
                ParticipantPosition::Long => true,
                ParticipantPosition::Short => false,
                _ => false
            }
        };
        let second_participant_position = {
            match participant_position {
                ParticipantPosition::Long => true,
                ParticipantPosition::Short => false,
                _ => false
            }
        };

        let valid_participant_position = {
            if !first_participant_position && second_participant_position {
                true
            }
            else if first_participant_position && !second_participant_position {
                true
            }
            else{false}
        };

        if !valid_participant_position {
            // Both predictions cannot not be same.
            return Err(Errors::PredictionCannotBeSame.into()); 
        }

        // Lets indicate that prediction has been completed by two participants
        deposit_account.made_prediction = true;
        deposit_account.second_participant = participant_position;

        // Lets maintain the pubkey of the second participant
        deposit_account.taker_auth = *ctx.accounts.deposit_auth.key;
        // Lets change the betting state to indicate limit of two participants has been met
        deposit_account.betting_state = 2;

        // step 1: deposit sol to participants(limited to two) vault
        let cpi_accounts = system_program::Transfer {
            from: deposit_auth.to_account_info(),
            to: ctx.accounts.sol_vault.to_account_info(),
        };

        let cpi = CpiContext::new(sys_program.to_account_info(), cpi_accounts);

        system_program::transfer(cpi, amount)?;

        Ok(())
    }

    // withdraw native sol 
    pub fn withdraw_participant_funds(ctx: Context<WithdrawParticipantFunds>, amount: u64) -> Result<()> {
        let valid_amount = {
            if amount > 0 {
                true
            }
            else{false}
        };
        // amount must be greater than zero
        if !valid_amount {
            return Err(Errors::AmountNotgreaterThanZero.into());
        }

        let deposit_account = &ctx.accounts.deposit_account;
        let deposit_auth = &ctx.accounts.deposit_auth;

        let valid_participant_key = {
            // This is a check to determine the person withdrawing participated in the prediction
            if deposit_account.deposit_auth.eq(deposit_auth.key) || deposit_account.taker_auth.eq(deposit_auth.key) {
                true
            }
            else{false}
        };

        // we only allow either first or second participants to withdraw since the two made the prediction
        if !valid_participant_key{
            return Err(Errors::WithdrawalDisAllowed.into());
        }

        // participant must have made a correct prediction and won it
        if !deposit_account.made_prediction{
            return Err(Errors::InvalidPrediction.into());
        }

        let valid_participant_winner = {
            // This is a check to determine the person withdrawing is the one who won the prediction
            if deposit_account.winner_auth.eq(deposit_auth.key) {
                true
            }
            else{false}
        };

        if !valid_participant_winner {
            // Invalid participant winner.
            return Err(Errors::InvalidWinner.into());
        }

        let valid_amount = {
            if amount == deposit_account.total_payout {
                true
            }
            else{false}
        };
        // withdrawal amount must be equal to total payout amount
        if !valid_amount {
            return Err(Errors::AmountNotEqualToTotalPayoutAmount.into());
        }

        let sys_program = &ctx.accounts.system_program;
        //let deposit_account = &ctx.accounts.deposit_account;
        let pda_auth = &mut ctx.accounts.pda_auth;
        let sol_vault = &mut ctx.accounts.sol_vault;

        let cpi_accounts = system_program::Transfer {
            from: sol_vault.to_account_info(),
            to: ctx.accounts.deposit_auth.to_account_info(),
        };

        let seeds = &[
            b"sol_vault",
            pda_auth.to_account_info().key.as_ref(),
            &[deposit_account.sol_vault_bump.unwrap()],
        ];

        let signer = &[&seeds[..]];

        let cpi = CpiContext::new_with_signer(sys_program.to_account_info(), cpi_accounts, signer);

        system_program::transfer(cpi, amount)?;

        Ok(())
    }

    pub fn process_prediction(ctx: Context<ProcessPrediction>, bet_fees: u64) -> Result<()> {
        let valid_amount = {
            if bet_fees > 0 {
                true
            }
            else{false}
        };
        // amount must be greater than zero
        if !valid_amount {
            return Err(Errors::AmountNotgreaterThanZero.into());
        }

        // Test Pyth oracle price feeds
        let price_feed = &ctx.accounts.pyth_price_feed_account;
        let current_timestamp1 = Clock::get()?.unix_timestamp;
        let current_price = price_feed
            .get_price_no_older_than(current_timestamp1, STALENESS_THRESHOLD)
            .ok_or(Errors::PythOffline)?;

        //let price  = current_price.price * 10^(current_price.expo as i64);
        /*
        let a = 10.0_f32;
        let b = a.powi(current_price.expo);
        let price = current_price.price as f32 * b;
        */
        let a = 10.0_f32;
        //let b: i32 = -8;
        let _expo = current_price.expo;
        //let c = a.powi(b) as f64;
        //let c = a.powi(b);
        //let c = a.powi(b);
        //let c = 10.0_f32.powi(-8); ***
        let c = a.powi(_expo);
        //let d: i64 = 2056422249;
        let d: i64 = current_price.price;
        //let d: i64 = 2056422249;
        //let f = d as f64 * c as f64;
        //let f = c as f64;
        //let g: f64 = f.trunc();
        //let e: u64 = f.trunc() as u64;
        //let price  = e;
        let f = c * d as f32;
        let price  = f;

        let deposit_account = &mut ctx.accounts.deposit_account;
        let pda_auth = &mut ctx.accounts.pda_auth;
        let sol_vault = &mut ctx.accounts.sol_vault;
        let sys_program = &ctx.accounts.system_program;

        // tests only
        deposit_account.pyth_price = current_price.price;
        deposit_account.pyth_expo = _expo;//current_price.expo;
        deposit_account._price = price;
        let price = deposit_account.strike_price;
        //

        let strike_price = deposit_account.strike_price;

        let first_participant_position = {
            match deposit_account.first_participant {
                ParticipantPosition::Long => true,
                ParticipantPosition::Short => false,
                _ => false
            }
        };
        let second_participant_position = {
            match deposit_account.second_participant {
                ParticipantPosition::Long => true,
                ParticipantPosition::Short => false,
                _ => false
            }
        };
        let valid_participant_position = {
            if !first_participant_position && second_participant_position {
                true
            }
            else if first_participant_position && !second_participant_position {
                true
            }
            else{false}
        };
        if !valid_participant_position {
            // Both predictions cannot not be same.
            return Err(Errors::PredictionCannotBeSame.into());
        }
        /*
        let winning_position_bool = {
            match winning_position {
                ParticipantPosition::Long => true,
                ParticipantPosition::Short => false,
                _ => false
            }
        };
        */

        // We are making an assumption that if the prices match then the long position was correct
        let winning_position_bool = {
            if price > 0 && strike_price == price as u64 {
                true
            }
            else {false}
        };

        let bet_amount = deposit_account.bet_amount;
        let taker_amount = deposit_account.taker_amount;

        let valid_amount = {
            if bet_amount + taker_amount > bet_fees  {
                true
            }
            else{false}
        };
        // bet_fees exceeds (bet_amount + taker_amount)
        if !valid_amount {
            return Err(Errors::InvalidWinningAmount.into());
        }
        
        let total_payout: u64 = bet_amount + taker_amount - bet_fees;
        let mut valid_position = false;
        // first_participant - deposit_account.deposit_auth
        // second_participant -  deposit_account.taker_auth
        /*
        if first_participant_position && winning_position_bool {
            deposit_account.winner_auth = deposit_account.deposit_auth;
            deposit_account.total_payout = total_payout;
            valid_position = true;
        }
        else if second_participant_position && winning_position_bool {
            deposit_account.winner_auth = deposit_account.taker_auth;
            deposit_account.total_payout = total_payout;
            valid_position = true;
        }
        else{}
        */

        // We are making an assumption that if the prices match then the long position was correct (i.e first_participant won)
        // else second_participant won
        if first_participant_position && winning_position_bool {
            deposit_account.winner_auth = deposit_account.deposit_auth;
            deposit_account.total_payout = total_payout;
            valid_position = true;
        }
        else {
            deposit_account.winner_auth = deposit_account.taker_auth;
            deposit_account.total_payout = total_payout;
            valid_position = true;
        }

        if valid_position {
            // step 1: deposit (bet_fees) sol to admin vault
            let cpi_accounts = system_program::Transfer {
                from: sol_vault.to_account_info(),
                to: ctx.accounts.admin_sol_vault.to_account_info(),
            };

            let seeds = &[
                b"sol_vault",
                pda_auth.to_account_info().key.as_ref(),
                &[deposit_account.sol_vault_bump.unwrap()],
            ];

            let signer = &[&seeds[..]];

            let cpi = CpiContext::new_with_signer(sys_program.to_account_info(), cpi_accounts, signer);

            system_program::transfer(cpi, bet_fees)?;
        }
        
        Ok(())
    }

    // admin (on behalf of house) withdraws native sol 
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let sys_program = &ctx.accounts.system_program;
        let deposit_account = &ctx.accounts.admin_deposit_account;
        let pda_auth = &mut ctx.accounts.admin_pda_auth;
        let sol_vault = &mut ctx.accounts.admin_sol_vault;

        let cpi_accounts = system_program::Transfer {
            from: sol_vault.to_account_info(),
            to: ctx.accounts.admin_auth.to_account_info(),
        };

        let seeds = &[
            b"admin_sol_vault",
            pda_auth.to_account_info().key.as_ref(),
            &[deposit_account.admin_sol_vault_bump.unwrap()],
        ];

        let signer = &[&seeds[..]];

        let cpi = CpiContext::new_with_signer(sys_program.to_account_info(), cpi_accounts, signer);

        system_program::transfer(cpi, amount)?;

        Ok(())
    }

}

#[derive(Accounts)]
pub struct Initialize<'info> {
    // Pyth Oracle price feeds accounts
    #[account(address = *program_id @ Errors::Unauthorized)]
    pub program: Signer<'info>,
    //#[account(mut)]
    //pub payer: Signer<'info>,
    #[account(init, payer = admin_auth, space = 8 + size_of::<AdminConfig>())]
    pub config: Account<'info, AdminConfig>,
    //
    #[account(init, payer = admin_auth, space = DepositBaseAdmin::LEN,
        constraint = !admin_deposit_account.is_initialized @ Errors::AccountAlreadyInitialized
    )]
    pub admin_deposit_account: Account<'info, DepositBaseAdmin>,
    #[account(seeds = [b"admin_auth", admin_deposit_account.key().as_ref()], bump)]
    /// CHECK: no need to check this.
    pub admin_pda_auth: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"admin_sol_vault", admin_pda_auth.key().as_ref()], bump)]
    pub admin_sol_vault: SystemAccount<'info>,
    #[account(mut)]
    pub admin_auth: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateBinaryOptions<'info> {
    #[account(init, payer = deposit_auth, space = BinaryOption::LEN)]
    pub deposit_account: Account<'info, BinaryOption>,
    #[account(seeds = [b"auth", deposit_account.key().as_ref()], bump)]
    /// CHECK: no need to check this.
    pub pda_auth: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"sol_vault", pda_auth.key().as_ref()], bump)]
    pub sol_vault: SystemAccount<'info>,
    #[account(mut)]
    pub deposit_auth: Signer<'info>,
    //admin accs
    #[account(mut,
        constraint = admin_deposit_account.is_initialized @ Errors::AccountNotInitialized
    )]
    pub admin_deposit_account: Account<'info, DepositBaseAdmin>,
    //
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptBinaryOptions<'info> {
    //admin accs
    #[account(mut,
        constraint = admin_deposit_account.is_initialized @ Errors::AccountNotInitialized
    )]
    pub admin_deposit_account: Account<'info, DepositBaseAdmin>,
    #[account(seeds = [b"admin_auth", admin_deposit_account.key().as_ref()], bump = admin_deposit_account.admin_auth_bump)]
    /// CHECK: no need to check this.
    pub admin_pda_auth: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"admin_sol_vault", admin_pda_auth.key().as_ref()], bump = admin_deposit_account.admin_sol_vault_bump.unwrap())]
    pub admin_sol_vault: SystemAccount<'info>,
    //admin accs
    #[account(mut,
        constraint = deposit_account.betting_state == 1 @ Errors::InvalidParticipantsLimit,
    )]
    pub deposit_account: Account<'info, BinaryOption>,
    #[account(seeds = [b"auth", deposit_account.key().as_ref()], bump = deposit_account.auth_bump)]
    /// CHECK: no need to check this.
    pub pda_auth: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"sol_vault", pda_auth.key().as_ref()], bump = deposit_account.sol_vault_bump.unwrap())]
    pub sol_vault: SystemAccount<'info>,
    #[account(mut)]
    pub deposit_auth: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawParticipantFunds<'info> {
    pub deposit_account: Account<'info, BinaryOption>,
    #[account(seeds = [b"auth", deposit_account.key().as_ref()], bump = deposit_account.auth_bump)]
    /// CHECK: no need to check this.
    pub pda_auth: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"sol_vault", pda_auth.key().as_ref()], bump = deposit_account.sol_vault_bump.unwrap())]
    pub sol_vault: SystemAccount<'info>,
    #[account(mut)]
    pub deposit_auth: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProcessPrediction<'info> {
    // Pyth Oracle price feeds accounts
    pub config: Account<'info, AdminConfig>,
    #[account(address = config.price_feed_id @ Errors::InvalidArgument)]
    pub pyth_price_feed_account: Account<'info, PriceFeed>,
    //#[account(address = config.collateral_price_feed_id @ Errors::InvalidArgument)]
    //pub pyth_collateral_account: Account<'info, PriceFeed>,
    //
    #[account(mut)]
    pub deposit_account: Account<'info, BinaryOption>,
    #[account(seeds = [b"auth", deposit_account.key().as_ref()], bump = deposit_account.auth_bump)]
    /// CHECK: no need to check this.
    pub pda_auth: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"sol_vault", pda_auth.key().as_ref()], bump = deposit_account.sol_vault_bump.unwrap())]
    pub sol_vault: SystemAccount<'info>,
    //admin accs
    #[account(mut,
        constraint = admin_deposit_account.is_initialized @ Errors::AccountNotInitialized
    )]
    pub admin_deposit_account: Account<'info, DepositBaseAdmin>,
    #[account(seeds = [b"admin_auth", admin_deposit_account.key().as_ref()], bump = admin_deposit_account.admin_auth_bump)]
    /// CHECK: no need to check this.
    pub admin_pda_auth: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"admin_sol_vault", admin_pda_auth.key().as_ref()], bump = admin_deposit_account.admin_sol_vault_bump.unwrap())]
    pub admin_sol_vault: SystemAccount<'info>,
    //admin accs
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(has_one = admin_auth)]
    pub admin_deposit_account: Account<'info, DepositBaseAdmin>,
    #[account(seeds = [b"admin_auth", admin_deposit_account.key().as_ref()], bump = admin_deposit_account.admin_auth_bump)]
    /// CHECK: no need to check this.
    pub admin_pda_auth: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"admin_sol_vault", admin_pda_auth.key().as_ref()], bump = admin_deposit_account.admin_sol_vault_bump.unwrap())]
    pub admin_sol_vault: SystemAccount<'info>,
    #[account(mut)]
    pub admin_auth: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct BinaryOption {
    pub deposit_auth: Pubkey,
    pub taker_auth: Pubkey,
    pub winner_auth: Pubkey,
    pub auth_bump: u8,
    pub sol_vault_bump: Option<u8>,
    pub bet_description: String,
    pub bet_amount: u64,
    pub taker_amount: u64,
    pub strike_price: u64,
    pub deposited_amount: u64,
    pub made_prediction: bool,
    pub total_payout: u64,
    // tests only
    pub pyth_price: i64,
    pub pyth_expo: i32,
    //pub _price: u64,
    pub _price: f32,
    //
    pub first_participant: ParticipantPosition,
    pub second_participant: ParticipantPosition,
    pub betting_state: u8,
}

const DISCRIMINATOR_LENGTH: usize = 8;
const PUBLIC_KEY_LENGTH: usize = 32;
const U64_LENGTH: usize = 8;
const U8_LENGTH: usize = 1;
const BOOL_LENGTH: usize = 1;
const OPTION_LENGTH: usize = 1; // 1 + (space(T))
const ENUM_LENGTH: usize = 1; // 1 + Largest Variant Size

impl BinaryOption {
    const LEN: usize = DISCRIMINATOR_LENGTH +
                       (PUBLIC_KEY_LENGTH * 3) +
                       (1 + U8_LENGTH * 3) +
                       DESCRIPTION_LENGTH +
                       (U64_LENGTH * 8) + //5
                       BOOL_LENGTH +
                       (ENUM_LENGTH + U8_LENGTH) +
                       (ENUM_LENGTH + U8_LENGTH);
}
#[account]
pub struct DepositBaseAdmin {
    pub admin_auth: Pubkey,
    pub admin_auth_bump: u8,
    pub admin_sol_vault_bump: Option<u8>,
    pub is_initialized: bool,
}

impl DepositBaseAdmin {
    const LEN: usize = DISCRIMINATOR_LENGTH +
                       PUBLIC_KEY_LENGTH +
                       U8_LENGTH +
                       1 + U8_LENGTH +
                       //(1 + 2 * U8_LENGTH) +
                       BOOL_LENGTH;
}

//Calculate the space for the enum. I just gave it value 1
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub enum ParticipantPosition {
    Long,
    Short,
    Unknown,
}
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub enum Participants {
    First,
    Second,
    Unknown,
}