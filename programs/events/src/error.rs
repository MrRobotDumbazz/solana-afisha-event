use anchor_lang::prelude::*;

#[error_code]
pub enum EventError {
    #[msg("Slug must be 1-32 chars of lowercase latin, digits and hyphens")]
    InvalidSlug,
    #[msg("Title must be 1-64 chars")]
    InvalidTitle,
    #[msg("Description must be at most 512 chars")]
    InvalidDescription,
    #[msg("Venue must be 1-64 chars")]
    InvalidVenue,
    #[msg("City must be 1-32 chars")]
    InvalidCity,
    #[msg("Image URI must be at most 128 chars")]
    InvalidImageUri,
    #[msg("Event must start in the future")]
    InvalidStartDate,
    #[msg("Event must end after it starts")]
    InvalidEndDate,
    #[msg("Capacity must be at least 1")]
    InvalidCapacity,
    #[msg("Capacity cannot be less than tickets already sold")]
    CapacityBelowSold,
    #[msg("Event is not active")]
    EventNotActive,
    #[msg("Event has already started")]
    EventAlreadyStarted,
    #[msg("Event has not ended yet")]
    EventNotEnded,
    #[msg("Invalid withdrawal amount")]
    InvalidAmount,
    #[msg("Event has already started, tickets cannot be purchased")]
    SalesClosed,
    #[msg("Event is sold out")]
    SoldOut,
    #[msg("Ticket belongs to a different event")]
    TicketEventMismatch,
    #[msg("Ticket has already been used")]
    TicketAlreadyUsed,
    #[msg("Check-in is not open yet")]
    CheckInNotOpen,
    #[msg("Check-in window has closed")]
    CheckInClosed,
    #[msg("Invalid ticket token account")]
    InvalidTicketAccount,
}
