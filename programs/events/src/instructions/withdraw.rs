use anchor_lang::prelude::*;

use crate::{
    constants::*, error::EventError, logs::FundsWithdrawn, state::Event, state::EventStatus,
};

#[derive(Accounts)]
#[instruction(slug: String)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub organizer: Signer<'info>,

    #[account(
        seeds = [EVENT_SEED, organizer.key().as_ref(), slug.as_bytes()],
        bump,
        has_one = organizer,
    )]
    pub event: Account<'info, Event>,

    #[account(mut, seeds = [VAULT_SEED, event.key().as_ref()], bump)]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_withdraw(ctx: Context<Withdraw>, _slug: String, amount: u64) -> Result<()> {
    let event = &ctx.accounts.event;
    let now = Clock::get()?.unix_timestamp;
    require!(
        event.status == EventStatus::Cancelled || now > event.ends_at,
        EventError::EventNotEnded
    );

    let rent_min = Rent::get()?.minimum_balance(0);
    let vault_lamports = ctx.accounts.vault.lamports();
    require!(
        amount > 0 && amount <= vault_lamports.saturating_sub(rent_min),
        EventError::InvalidAmount
    );

    let event_key = event.key();
    let vault_seeds: &[&[u8]] = &[VAULT_SEED, event_key.as_ref(), &[ctx.bumps.vault]];
    let signer_seeds = [vault_seeds];
    anchor_lang::system_program::transfer(
        CpiContext::new_with_signer(
            anchor_lang::system_program::ID,
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.organizer.to_account_info(),
            },
            &signer_seeds,
        ),
        amount,
    )?;

    emit!(FundsWithdrawn {
        event: event.key(),
        organizer: ctx.accounts.organizer.key(),
        amount,
    });

    Ok(())
}
