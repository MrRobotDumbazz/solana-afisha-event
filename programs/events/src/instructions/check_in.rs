use anchor_lang::prelude::*;

use crate::{
    constants::*, error::EventError, logs::TicketCheckedIn, state::Event, state::EventStatus,
    state::Ticket, state::TicketStatus,
};

#[derive(Accounts)]
#[instruction(slug: String)]
pub struct CheckIn<'info> {
    pub organizer: Signer<'info>,

    #[account(
        seeds = [EVENT_SEED, organizer.key().as_ref(), slug.as_bytes()],
        bump,
        has_one = organizer,
    )]
    pub event: Account<'info, Event>,

    #[account(
        mut,
        has_one = event @ EventError::TicketEventMismatch,
        constraint = ticket.status == TicketStatus::Valid @ EventError::TicketAlreadyUsed,
    )]
    pub ticket: Account<'info, Ticket>,
}

pub fn handle_check_in(ctx: Context<CheckIn>, _slug: String) -> Result<()> {
    let event = &ctx.accounts.event;
    let now = Clock::get()?.unix_timestamp;

    require!(
        now >= event.starts_at - CHECK_IN_OPEN_BEFORE_START_SECS,
        EventError::CheckInNotOpen
    );
    require!(
        now <= event.ends_at + CHECK_IN_GRACE_AFTER_END_SECS,
        EventError::CheckInClosed
    );
    require!(
        event.status == EventStatus::Active,
        EventError::EventNotActive
    );

    let ticket = &mut ctx.accounts.ticket;
    ticket.status = TicketStatus::Used;
    ticket.checked_in_at = now;

    emit!(TicketCheckedIn {
        event: event.key(),
        ticket: ticket.key(),
        buyer: ticket.buyer,
        checked_in_at: now,
    });

    Ok(())
}
