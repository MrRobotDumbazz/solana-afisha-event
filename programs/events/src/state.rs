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
