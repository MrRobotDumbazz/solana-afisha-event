use anchor_lang::prelude::*;

use crate::{constants::*, logs::EventCancelled, state::Event, state::EventStatus};

#[derive(Accounts)]
#[instruction(slug: String)]
pub struct CancelEvent<'info> {
    #[account(mut)]
    pub organizer: Signer<'info>,

    #[account(
        mut,
        seeds = [EVENT_SEED, organizer.key().as_ref(), slug.as_bytes()],
        bump,
        has_one = organizer,
        constraint = event.status == EventStatus::Active,
    )]
    pub event: Account<'info, Event>,
}

pub fn handle_cancel_event(ctx: Context<CancelEvent>, _slug: String) -> Result<()> {
    ctx.accounts.event.status = EventStatus::Cancelled;

    emit!(EventCancelled {
        event: ctx.accounts.event.key(),
    });

    Ok(())
}
