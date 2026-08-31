use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::initialize_mint2;
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_2022::InitializeMint2;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_2022_extensions::metadata_pointer::metadata_pointer_initialize;
use anchor_spl::token_2022_extensions::metadata_pointer::MetadataPointerInitialize;
use anchor_spl::token_2022_extensions::token_metadata::token_metadata_initialize;
use anchor_spl::token_2022_extensions::token_metadata::TokenMetadataInitialize;

use crate::{
    constants::*, error::EventError, logs::TicketPurchased, state::truncate_bytes, state::Event,
    state::EventStatus, state::Ticket, state::TicketStatus,
};

pub fn ticket_mint_init_space() -> usize {
    use anchor_spl::token_2022::spl_token_2022::extension::ExtensionType;
    use anchor_spl::token_2022::spl_token_2022::state::Mint as SplMint;

    ExtensionType::try_calculate_account_len::<SplMint>(&[ExtensionType::MetadataPointer]).unwrap()
}

pub fn ticket_mint_full_space(name: &str, symbol: &str, uri: &str) -> usize {
    let tlv_header = 4;
    let metadata_payload = 33 + 32 + 4 + name.len() + 4 + symbol.len() + 4 + uri.len() + 4;
    ticket_mint_init_space() + tlv_header + metadata_payload
}

#[derive(Accounts)]
pub struct BuyTicket<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        constraint = event.status == EventStatus::Active @ EventError::EventNotActive,
    )]
    pub event: Account<'info, Event>,

    #[account(mut, seeds = [VAULT_SEED, event.key().as_ref()], bump)]
    pub vault: SystemAccount<'info>,

    /// CHECK: token-2022 mint PDA ["mint", event, buyer], created manually in
    /// the handler with space reserved for the TokenMetadata extension.
    #[account(
        mut,
        seeds = [MINT_SEED, event.key().as_ref(), buyer.key().as_ref()],
        bump
    )]
    pub mint: UncheckedAccount<'info>,

    #[account(
        init,
        payer = buyer,
        space = 8 + Ticket::INIT_SPACE,
        seeds = [TICKET_SEED, event.key().as_ref(), buyer.key().as_ref()],
        bump,
    )]
    pub ticket: Account<'info, Ticket>,

    /// CHECK: buyer's associated token account for the mint, derived address
    /// verified in the handler.
    #[account(mut)]
    pub buyer_ata: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_buy_ticket(ctx: Context<BuyTicket>) -> Result<()> {
    let event = &mut ctx.accounts.event;
    let now = Clock::get()?.unix_timestamp;
    require!(now < event.starts_at, EventError::SalesClosed);
    require!(event.tickets_sold < event.capacity, EventError::SoldOut);

    let buyer_key = ctx.accounts.buyer.key;
    let mint_key = ctx.accounts.mint.key();
    let token_program_key = ctx.accounts.token_program.key();

    let expected_ata = anchor_spl::associated_token::get_associated_token_address_with_program_id(
        &buyer_key,
        &mint_key,
        &token_program_key,
    );
    require_keys_eq!(
        ctx.accounts.buyer_ata.key(),
        expected_ata,
        EventError::InvalidTicketAccount
    );

    let price = event.ticket_price_lamports;
    if price > 0 {
        anchor_lang::system_program::transfer(
            CpiContext::new(
                anchor_lang::system_program::ID,
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                },
            ),
            price,
        )?;
    }

    let mint_info = ctx.accounts.mint.to_account_info();

    let event_key = event.key();
    let mint_seeds: &[&[u8]] = &[
        MINT_SEED,
        event_key.as_ref(),
        buyer_key.as_ref(),
        &[ctx.bumps.mint],
    ];
    let signer_seeds = [mint_seeds];

    let ticket_number = event.tickets_sold + 1;
    let name = format!("{} #{}", event.title, ticket_number);
    let name = truncate_bytes(&name, TICKET_META_NAME_MAX).to_string();
    let symbol = truncate_bytes(TICKET_META_SYMBOL, TICKET_META_SYMBOL_MAX).to_string();
    let uri = truncate_bytes(&event.image_uri, TICKET_META_URI_MAX).to_string();

    let space = ticket_mint_init_space() as u64;
    let lamports = Rent::get()?.minimum_balance(ticket_mint_full_space(&name, &symbol, &uri));
    anchor_lang::system_program::create_account(
        CpiContext::new_with_signer(
            anchor_lang::system_program::ID,
            anchor_lang::system_program::CreateAccount {
                from: ctx.accounts.buyer.to_account_info(),
                to: mint_info.clone(),
            },
            &signer_seeds,
        ),
        lamports,
        space,
        &token_program_key,
    )?;

    metadata_pointer_initialize(
        CpiContext::new(
            token_program_key,
            MetadataPointerInitialize {
                token_program_id: ctx.accounts.token_program.to_account_info(),
                mint: mint_info.clone(),
            },
        ),
        None,
        Some(mint_key),
    )?;

    initialize_mint2(
        CpiContext::new(
            token_program_key,
            InitializeMint2 {
                mint: mint_info.clone(),
            },
        ),
        0,
        &mint_key,
        None,
    )?;

    token_metadata_initialize(
        CpiContext::new_with_signer(
            token_program_key,
            TokenMetadataInitialize {
                program_id: ctx.accounts.token_program.to_account_info(),
                metadata: mint_info.clone(),
                update_authority: mint_info.clone(),
                mint_authority: mint_info.clone(),
                mint: mint_info.clone(),
            },
            &signer_seeds,
        ),
        name,
        symbol,
        uri,
    )?;

    let ata_info = ctx.accounts.buyer_ata.to_account_info();
    let create_ata_ix =
        spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            &buyer_key,
            &buyer_key,
            &mint_key,
            &token_program_key,
        );
    invoke(
        &create_ata_ix,
        &[
            ctx.accounts.buyer.to_account_info(),
            ata_info.clone(),
            ctx.accounts.buyer.to_account_info(),
            mint_info.clone(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
    )?;

    let ata_key = ctx.accounts.buyer_ata.key();
    let mint_to_ix = spl_token_2022::instruction::mint_to(
        &token_program_key,
        &mint_key,
        &ata_key,
        &mint_key,
        &[],
        1,
    )?;
    invoke_signed(
        &mint_to_ix,
        &[mint_info.clone(), ata_info.clone(), mint_info.clone()],
        &signer_seeds,
    )?;

    let ticket = &mut ctx.accounts.ticket;
    ticket.event = event.key();
    ticket.buyer = *buyer_key;
    ticket.mint = mint_key;
    ticket.status = TicketStatus::Valid;
    ticket.checked_in_at = 0;

    event.tickets_sold = ticket_number;

    emit!(TicketPurchased {
        event: event.key(),
        buyer: ticket.buyer,
        ticket: ticket.key(),
        mint: ticket.mint,
        ticket_number,
        price_lamports: price,
    });

    Ok(())
}
