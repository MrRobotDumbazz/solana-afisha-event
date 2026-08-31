use anchor_lang::prelude::*;

use crate::{
    constants::*, logs::EventCreated, state::validate_slug, state::Event, state::EventParams,
    state::EventStatus,
};

#[derive(Accounts)]
#[instruction(slug: String)]
pub struct InitEvent<'info> {
    #[account(mut)]
    pub organizer: Signer<'info>,

    #[account(
        init,
        payer = organizer,
        space = 8 + Event::INIT_SPACE,
        seeds = [EVENT_SEED, organizer.key().as_ref(), slug.as_bytes()],
        bump,
    )]
    pub event: Account<'info, Event>,

    /// CHECK: zero-byte escrow PDA owned by the system program, created via
    /// create_account CPI below. Carries SOL only, no data.
    #[account(mut, seeds = [VAULT_SEED, event.key().as_ref()], bump)]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_init_event(ctx: Context<InitEvent>, slug: String, params: EventParams) -> Result<()> {
    validate_slug(&slug)?;
    params.validate()?;

    let event = &mut ctx.accounts.event;
    event.organizer = ctx.accounts.organizer.key();
    event.slug = slug.clone();
    event.title = params.title.clone();
    event.description = params.description.clone();
    event.venue = params.venue.clone();
    event.city = params.city.clone();
    event.image_uri = params.image_uri.clone();
    event.starts_at = params.starts_at;
    event.ends_at = params.ends_at;
    event.ticket_price_lamports = params.ticket_price_lamports;
    event.capacity = params.capacity;
    event.tickets_sold = 0;
    event.hot_sale = params.hot_sale;
    event.status = EventStatus::Active;

    let rent_min = Rent::get()?.minimum_balance(0);
    let event_key = event.key();
    let vault_seeds: &[&[u8]] = &[VAULT_SEED, event_key.as_ref(), &[ctx.bumps.vault]];
    let signer_seeds = [vault_seeds];
    anchor_lang::system_program::create_account(
        CpiContext::new_with_signer(
            anchor_lang::system_program::ID,
            anchor_lang::system_program::CreateAccount {
                from: ctx.accounts.organizer.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
            &signer_seeds,
        ),
        rent_min,
        0,
        &anchor_lang::system_program::ID,
    )?;

    emit!(EventCreated {
        event: event.key(),
        organizer: event.organizer,
        slug,
        title: params.title,
        city: params.city,
        starts_at: event.starts_at,
        ends_at: event.ends_at,
        ticket_price_lamports: event.ticket_price_lamports,
        capacity: event.capacity,
        hot_sale: event.hot_sale,
    });

    Ok(())
}
