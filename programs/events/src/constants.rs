use anchor_lang::prelude::*;

#[constant]
pub const EVENT_SEED: &[u8] = b"event";

#[constant]
pub const VAULT_SEED: &[u8] = b"vault";

#[constant]
pub const TICKET_SEED: &[u8] = b"ticket";

#[constant]
pub const MINT_SEED: &[u8] = b"mint";

#[constant]
pub const SALE_SEED: &[u8] = b"sale";

#[constant]
pub const QUEUE_SEED: &[u8] = b"queue";

pub const SLUG_MAX: usize = 32;
pub const TITLE_MAX: usize = 64;
pub const DESCRIPTION_MAX: usize = 512;
pub const VENUE_MAX: usize = 64;
pub const CITY_MAX: usize = 32;
pub const IMAGE_URI_MAX: usize = 128;

pub const TICKET_META_NAME_MAX: usize = 64;
pub const TICKET_META_SYMBOL_MAX: usize = 8;
pub const TICKET_META_SYMBOL: &str = "AFISHA";
pub const TICKET_META_URI_MAX: usize = 128;

pub const CHECK_IN_OPEN_BEFORE_START_SECS: i64 = 3600;
pub const CHECK_IN_GRACE_AFTER_END_SECS: i64 = 24 * 3600;
