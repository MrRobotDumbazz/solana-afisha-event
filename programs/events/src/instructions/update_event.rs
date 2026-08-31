use anchor_lang::prelude::*;

use crate::{
    constants::*, error::EventError, logs::EventUpdated, state::Event, state::EventParams,
    state::EventStatus,
};

#[derive(Accounts)]
#[instruction(slug: String)]
pub struct UpdateEvent<'info> {
    #[account(mut)]
    pub organizer: Signer<'info>,

    #[account(
        mut,
        seeds = [EVENT_SEED, organizer.key().as_ref(), slug.as_bytes()],
        bump,
        has_one = organizer,
        constraint = event.status == EventStatus::Active @ EventError::EventNotActive,
    )]
    pub event: Account<'info, Event>,
}

pub fn handle_update_event(
    ctx: Context<UpdateEvent>,
    _slug: String,
    params: EventParams,
) -> Result<()> {
    params.validate()?;

    let event = &mut ctx.accounts.event;
    require!(
        params.capacity >= event.tickets_sold,
        EventError::CapacityBelowSold
    );

    event.title = params.title.clone();
    event.description = params.description.clone();
    event.venue = params.venue.clone();
    event.city = params.city.clone();
    event.image_uri = params.image_uri.clone();
    event.starts_at = params.starts_at;
    event.ends_at = params.ends_at;
    event.ticket_price_lamports = params.ticket_price_lamports;
    event.capacity = params.capacity;
    event.hot_sale = params.hot_sale;

    emit!(EventUpdated {
        event: event.key(),
        title: params.title,
        starts_at: event.starts_at,
        ends_at: event.ends_at,
        ticket_price_lamports: event.ticket_price_lamports,
        capacity: event.capacity,
    });

    Ok(())
}
