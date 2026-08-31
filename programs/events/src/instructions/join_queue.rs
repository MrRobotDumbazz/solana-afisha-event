use anchor_lang::prelude::*;

use crate::{
    constants::*, error::EventError, logs::QueueJoined, state::Event, state::EventStatus,
    state::QueueEntry, state::QueueStatus, state::SaleState,
};

#[derive(Accounts)]
pub struct JoinQueue<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        constraint = event.status == EventStatus::Active @ EventError::EventNotActive,
        constraint = event.hot_sale @ EventError::NotHotSale,
    )]
    pub event: Account<'info, Event>,

    #[account(
        mut,
        seeds = [SALE_SEED, event.key().as_ref()],
        bump,
        has_one = event @ EventError::EntryMismatch,
        constraint = !sale.settled @ EventError::RandomnessNotSettled,
    )]
    pub sale: Account<'info, SaleState>,

    #[account(
        init,
        payer = buyer,
        space = 8 + QueueEntry::INIT_SPACE,
        seeds = [QUEUE_SEED, event.key().as_ref(), buyer.key().as_ref()],
        bump,
    )]
    pub entry: Account<'info, QueueEntry>,

    #[account(mut, seeds = [VAULT_SEED, event.key().as_ref()], bump)]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_join_queue(ctx: Context<JoinQueue>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let sale = &mut ctx.accounts.sale;
    require!(
        now >= sale.registration_start && now < sale.registration_end,
        EventError::RegistrationClosed
    );

    if sale.stake_lamports > 0 {
        anchor_lang::system_program::transfer(
            CpiContext::new(
                anchor_lang::system_program::ID,
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                },
            ),
            sale.stake_lamports,
        )?;
    }

    let position = sale.total_entries;
    sale.total_entries += 1;

    let entry = &mut ctx.accounts.entry;
    entry.event = ctx.accounts.event.key();
    entry.buyer = ctx.accounts.buyer.key();
    entry.position = position;
    entry.stake_lamports = sale.stake_lamports;
    entry.status = QueueStatus::Staked;

    emit!(QueueJoined {
        event: entry.event,
        buyer: entry.buyer,
        entry: entry.key(),
        position,
        total_entries: sale.total_entries,
    });

    Ok(())
}
