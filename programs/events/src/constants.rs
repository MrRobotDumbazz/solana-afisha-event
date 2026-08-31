use anchor_lang::prelude::*;

#[constant]
pub const EVENT_SEED: &[u8] = b"event";

#[constant]
pub const VAULT_SEED: &[u8] = b"vault";

pub const SLUG_MAX: usize = 32;
pub const TITLE_MAX: usize = 64;
pub const DESCRIPTION_MAX: usize = 512;
pub const VENUE_MAX: usize = 64;
pub const CITY_MAX: usize = 32;
pub const IMAGE_URI_MAX: usize = 128;
