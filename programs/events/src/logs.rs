use anchor_lang::prelude::*;

#[event]
pub struct EventCreated {
    pub event: Pubkey,
    pub organizer: Pubkey,
    pub slug: String,
    pub title: String,
    pub city: String,
    pub starts_at: i64,
    pub ends_at: i64,
    pub ticket_price_lamports: u64,
    pub capacity: u32,
    pub hot_sale: bool,
}

#[event]
pub struct EventUpdated {
    pub event: Pubkey,
    pub title: String,
    pub starts_at: i64,
    pub ends_at: i64,
    pub ticket_price_lamports: u64,
    pub capacity: u32,
}

#[event]
pub struct EventCancelled {
    pub event: Pubkey,
}

#[event]
pub struct FundsWithdrawn {
    pub event: Pubkey,
    pub organizer: Pubkey,
    pub amount: u64,
}

#[event]
pub struct TicketPurchased {
    pub event: Pubkey,
    pub buyer: Pubkey,
    pub ticket: Pubkey,
    pub mint: Pubkey,
    pub ticket_number: u32,
    pub price_lamports: u64,
}

#[event]
pub struct TicketCheckedIn {
    pub event: Pubkey,
    pub ticket: Pubkey,
    pub buyer: Pubkey,
    pub checked_in_at: i64,
}

#[event]
pub struct SaleConfigured {
    pub event: Pubkey,
    pub stake_lamports: u64,
    pub window_size: u32,
    pub round_duration_secs: i64,
}

#[event]
pub struct QueueJoined {
    pub event: Pubkey,
    pub buyer: Pubkey,
    pub entry: Pubkey,
    pub position: u32,
    pub total_entries: u32,
}

#[event]
pub struct RandomnessSettled {
    pub event: Pubkey,
    pub randomness: u64,
}

#[event]
pub struct StakeSettled {
    pub event: Pubkey,
    pub entry: Pubkey,
    pub buyer: Pubkey,
    pub refunded: bool,
    pub amount: u64,
}
