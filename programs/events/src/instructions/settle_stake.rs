use anchor_lang::prelude::*;

use crate::{
    constants::*, error::EventError, logs::StakeSettled, state::Event, state::EventStatus,
    state::QueueEntry, state::QueueStatus, state::SaleState,
};

#[derive(Accounts)]
pub struct SettleStake<'info> {
    pub caller: Signer<'info>,

    #[account(constraint = event.hot_sale @ EventError::NotHotSale)]
    pub event: Account<'info, Event>,

    #[account(
        mut,
        seeds = [SALE_SEED, event.key().as_ref()],
        bump,
        has_one = event @ EventError::EntryMismatch,
    )]
    pub sale: Account<'info, SaleState>,

    #[account(
        mut,
        seeds = [QUEUE_SEED, event.key().as_ref(), entry_buyer.key().as_ref()],
        bump,
        has_one = event @ EventError::EntryMismatch,
        constraint = entry.buyer == entry_buyer.key() @ EventError::EntryMismatch,
        constraint = entry.status == QueueStatus::Staked @ EventError::EntryNotStaked,
    )]
    pub entry: Account<'info, QueueEntry>,

    /// CHECK: payout target, must match entry.buyer
    #[account(mut)]
    pub entry_buyer: UncheckedAccount<'info>,

    #[account(mut, seeds = [VAULT_SEED, event.key().as_ref()], bump)]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_settle_stake(ctx: Context<SettleStake>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let event = &ctx.accounts.event;
    let sale = &mut ctx.accounts.sale;
    let entry = &mut ctx.accounts.entry;

    let sale_over = event.status == EventStatus::Cancelled
        || now >= event.starts_at
        || event.tickets_sold == event.capacity;
    require!(sale_over, EventError::SettleTooEarly);

    let forfeit =
        event.status != EventStatus::Cancelled && event.tickets_sold != event.capacity && {
            let round = sale.round_of(sale.effective_position(entry));
            let (_, round_end) = sale.round_bounds(round);
            now >= round_end
        };

    if !forfeit {
        if entry.stake_lamports > 0 {
            let event_key = event.key();
            let seeds: &[&[u8]] = &[VAULT_SEED, event_key.as_ref(), &[ctx.bumps.vault]];
            let signer_seeds = [seeds];
            anchor_lang::system_program::transfer(
                CpiContext::new_with_signer(
                    anchor_lang::system_program::ID,
                    anchor_lang::system_program::Transfer {
                        from: ctx.accounts.vault.to_account_info(),
                        to: ctx.accounts.entry_buyer.to_account_info(),
                    },
                    &signer_seeds,
                ),
                entry.stake_lamports,
            )?;
        }
        entry.status = QueueStatus::Settled;
        sale.settled_count += 1;
    } else {
        entry.status = QueueStatus::Forfeited;
        sale.forfeited_count += 1;
    }

    emit!(StakeSettled {
        event: event.key(),
        entry: entry.key(),
        buyer: entry.buyer,
        refunded: !forfeit,
        amount: entry.stake_lamports,
    });

    Ok(())
}
