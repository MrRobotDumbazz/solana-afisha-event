use anchor_lang::prelude::*;

use crate::{
    constants::*, error::EventError, logs::RandomnessSettled, state::Event, state::SaleState,
};

#[derive(Accounts)]
pub struct SettleRandomness<'info> {
    pub caller: Signer<'info>,

    #[account(constraint = event.hot_sale @ EventError::NotHotSale)]
    pub event: Account<'info, Event>,

    #[account(
        mut,
        seeds = [SALE_SEED, event.key().as_ref()],
        bump,
        has_one = event @ EventError::EntryMismatch,
        constraint = !sale.settled @ EventError::AlreadySettled,
    )]
    pub sale: Account<'info, SaleState>,

    /// CHECK: SlotHashes sysvar, address constrained; data parsed manually.
    #[account(address = solana_sdk_ids::sysvar::slot_hashes::ID)]
    pub slot_hashes: UncheckedAccount<'info>,
}

pub fn handle_settle_randomness(ctx: Context<SettleRandomness>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let sale = &mut ctx.accounts.sale;
    require!(now >= sale.reveal_at, EventError::RandomnessNotSettled);

    let data = ctx.accounts.slot_hashes.try_borrow_data()?;
    require!(data.len() >= 48, EventError::RandomnessNotSettled);
    let mut count_bytes = [0u8; 8];
    count_bytes.copy_from_slice(&data[0..8]);
    let count = u64::from_le_bytes(count_bytes);
    require!(count > 0, EventError::RandomnessNotSettled);

    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(&data[16..48]);
    let mut randomness_bytes = [0u8; 8];
    randomness_bytes.copy_from_slice(&hash_bytes[0..8]);
    let randomness = u64::from_le_bytes(randomness_bytes);

    sale.randomness = randomness;
    sale.settled = true;

    emit!(RandomnessSettled {
        event: sale.event,
        randomness,
    });

    Ok(())
}
