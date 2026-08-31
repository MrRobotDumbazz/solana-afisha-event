pub mod constants;
pub mod error;
pub mod instructions;
pub mod logs;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("7J6VC2HsTxBCBMc94FbcfT2NcN2bmSK5nhjejeuL4g8Y");

#[program]
pub mod events {
    use super::*;

    pub fn init_event(ctx: Context<InitEvent>, slug: String, params: EventParams) -> Result<()> {
        instructions::init_event::handle_init_event(ctx, slug, params)
    }

    pub fn update_event(
        ctx: Context<UpdateEvent>,
        slug: String,
        params: EventParams,
    ) -> Result<()> {
        instructions::update_event::handle_update_event(ctx, slug, params)
    }

    pub fn cancel_event(ctx: Context<CancelEvent>, slug: String) -> Result<()> {
        instructions::cancel_event::handle_cancel_event(ctx, slug)
    }

    pub fn withdraw(ctx: Context<Withdraw>, slug: String, amount: u64) -> Result<()> {
        instructions::withdraw::handle_withdraw(ctx, slug, amount)
    }
}
