use anchor_lang::prelude::*;

declare_id!("7J6VC2HsTxBCBMc94FbcfT2NcN2bmSK5nhjejeuL4g8Y");

#[program]
pub mod events {
    use super::*;

    pub fn health(_ctx: Context<Health>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Health {}
