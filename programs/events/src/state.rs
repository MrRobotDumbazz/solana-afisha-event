use anchor_lang::prelude::*;

use crate::{constants::*, error::EventError};

#[account]
#[derive(InitSpace)]
pub struct Event {
    pub organizer: Pubkey,
    #[max_len(SLUG_MAX)]
    pub slug: String,
    #[max_len(TITLE_MAX)]
    pub title: String,
    #[max_len(DESCRIPTION_MAX)]
    pub description: String,
    #[max_len(VENUE_MAX)]
    pub venue: String,
    #[max_len(CITY_MAX)]
    pub city: String,
    #[max_len(IMAGE_URI_MAX)]
    pub image_uri: String,
    pub starts_at: i64,
    pub ends_at: i64,
    pub ticket_price_lamports: u64,
    pub capacity: u32,
    pub tickets_sold: u32,
    pub hot_sale: bool,
    pub status: EventStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum EventStatus {
    Active,
    Cancelled,
}

#[account]
#[derive(InitSpace)]
pub struct Ticket {
    pub event: Pubkey,
    pub buyer: Pubkey,
    pub mint: Pubkey,
    pub status: TicketStatus,
    pub checked_in_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum TicketStatus {
    Valid,
    Used,
}

#[account]
#[derive(InitSpace)]
pub struct SaleState {
    pub event: Pubkey,
    pub registration_start: i64,
    pub registration_end: i64,
    pub reveal_at: i64,
    pub claim_start: i64,
    pub round_duration_secs: i64,
    pub stake_lamports: u64,
    pub window_size: u32,
    pub total_entries: u32,
    pub randomness: u64,
    pub settled: bool,
    pub claimed: u32,
    pub settled_count: u32,
    pub forfeited_count: u32,
    pub bump: u8,
}

impl SaleState {
    pub fn pending(&self) -> u32 {
        self.total_entries
            .saturating_sub(self.claimed)
            .saturating_sub(self.settled_count)
            .saturating_sub(self.forfeited_count)
    }

    pub fn effective_position(&self, entry: &QueueEntry) -> u32 {
        let total = self.total_entries as u64;
        if total == 0 {
            return 0;
        }
        ((self.randomness % total) + entry.position as u64 % total) as u32
    }

    pub fn round_of(&self, effective_position: u32) -> u64 {
        if self.window_size == 0 {
            return 0;
        }
        effective_position as u64 / self.window_size as u64
    }

    pub fn round_bounds(&self, round: u64) -> (i64, i64) {
        let start = self
            .claim_start
            .saturating_add((round.saturating_mul(self.round_duration_secs as u64)) as i64);
        (start, start.saturating_add(self.round_duration_secs))
    }
}

#[account]
#[derive(InitSpace)]
pub struct QueueEntry {
    pub event: Pubkey,
    pub buyer: Pubkey,
    pub position: u32,
    pub stake_lamports: u64,
    pub status: QueueStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum QueueStatus {
    Staked,
    Claimed,
    Settled,
    Forfeited,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct SaleParams {
    pub registration_start: i64,
    pub registration_end: i64,
    pub reveal_at: i64,
    pub claim_start: i64,
    pub round_duration_secs: i64,
    pub stake_lamports: u64,
    pub window_size: u32,
}

impl SaleParams {
    pub fn validate(&self, now: i64) -> Result<()> {
        require!(
            self.registration_start >= now,
            EventError::InvalidSalePhases
        );
        require!(
            self.registration_end > self.registration_start,
            EventError::InvalidSalePhases
        );
        require!(
            self.reveal_at >= self.registration_end,
            EventError::InvalidSalePhases
        );
        require!(
            self.claim_start >= self.reveal_at,
            EventError::InvalidSalePhases
        );
        require!(
            self.round_duration_secs >= 60,
            EventError::InvalidSalePhases
        );
        require!(self.window_size >= 1, EventError::InvalidSalePhases);
        Ok(())
    }
}

pub fn truncate_bytes(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        let mut end = max;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        &s[..end]
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct EventParams {
    pub title: String,
    pub description: String,
    pub venue: String,
    pub city: String,
    pub image_uri: String,
    pub starts_at: i64,
    pub ends_at: i64,
    pub ticket_price_lamports: u64,
    pub capacity: u32,
    pub hot_sale: bool,
}

pub fn validate_slug(slug: &str) -> Result<()> {
    require!(
        !slug.is_empty() && slug.len() <= SLUG_MAX,
        EventError::InvalidSlug
    );
    require!(
        slug.bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'-')),
        EventError::InvalidSlug
    );
    Ok(())
}

impl EventParams {
    pub fn validate(&self) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        require!(
            !self.title.is_empty() && self.title.len() <= TITLE_MAX,
            EventError::InvalidTitle
        );
        require!(
            self.description.len() <= DESCRIPTION_MAX,
            EventError::InvalidDescription
        );
        require!(
            !self.venue.is_empty() && self.venue.len() <= VENUE_MAX,
            EventError::InvalidVenue
        );
        require!(
            !self.city.is_empty() && self.city.len() <= CITY_MAX,
            EventError::InvalidCity
        );
        require!(
            self.image_uri.len() <= IMAGE_URI_MAX,
            EventError::InvalidImageUri
        );
        require!(self.starts_at > now, EventError::InvalidStartDate);
        require!(self.ends_at > self.starts_at, EventError::InvalidEndDate);
        require!(self.capacity >= 1, EventError::InvalidCapacity);
        Ok(())
    }
}
