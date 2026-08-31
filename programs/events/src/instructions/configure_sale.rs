use anchor_lang::prelude::*;

use crate::{
    constants::*, error::EventError, logs::SaleConfigured, state::Event, state::EventStatus,
    state::SaleParams, state::SaleState,
};

#[derive(Accounts)]
#[instruction(slug: String, params: SaleParams)]
pub struct ConfigureSale<'info> {
    #[account(mut)]
    pub organizer: Signer<'info>,

    #[account(
        seeds = [EVENT_SEED, organizer.key().as_ref(), slug.as_bytes()],
        bump,
        has_one = organizer,
        constraint = event.status == EventStatus::Active @ EventError::EventNotActive,
        constraint = event.hot_sale @ EventError::NotHotSale,
    )]
    pub event: Account<'info, Event>,

    #[account(
        init,
        payer = organizer,
        space = 8 + SaleState::INIT_SPACE,
        seeds = [SALE_SEED, event.key().as_ref()],
        bump,
    )]
    pub sale: Account<'info, SaleState>,

    pub system_program: Program<'info, System>,
}

pub fn handle_configure_sale(
    ctx: Context<ConfigureSale>,
    _slug: String,
    params: SaleParams,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    params.validate(now)?;

    let sale = &mut ctx.accounts.sale;
    sale.event = ctx.accounts.event.key();
    sale.registration_start = params.registration_start;
    sale.registration_end = params.registration_end;
    sale.reveal_at = params.reveal_at;
    sale.claim_start = params.claim_start;
    sale.round_duration_secs = params.round_duration_secs;
    sale.stake_lamports = params.stake_lamports;
    sale.window_size = params.window_size;
    sale.total_entries = 0;
    sale.randomness = 0;
    sale.settled = false;
    sale.claimed = 0;
    sale.settled_count = 0;
    sale.forfeited_count = 0;
    sale.bump = ctx.bumps.sale;

    emit!(SaleConfigured {
        event: sale.event,
        stake_lamports: sale.stake_lamports,
        window_size: sale.window_size,
        round_duration_secs: sale.round_duration_secs,
    });

    Ok(())
}
