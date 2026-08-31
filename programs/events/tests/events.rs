use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use litesvm::types::TransactionResult;
use litesvm::LiteSVM;
use sha2::Digest;
use sha2::Sha256;
use solana_clock::Clock;
use solana_instruction::account_meta::AccountMeta;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::pubkey;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

use events::state::Event;
use events::state::EventParams;
use events::state::EventStatus;
use events::state::TicketStatus;

const PROGRAM_ID: Pubkey = pubkey!("7J6VC2HsTxBCBMc94FbcfT2NcN2bmSK5nhjejeuL4g8Y");
const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
const LAMPORTS_FOR_TEST: u64 = 1_000_000_000_000;
const SLUG: &str = "solana-break-2026";

struct Env {
    svm: LiteSVM,
    payer: Keypair,
}

fn setup() -> Env {
    let mut svm = LiteSVM::new();
    svm.add_program(
        PROGRAM_ID,
        include_bytes!("../../../target/sbpf-solana-solana/release/events.so"),
    )
    .unwrap();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), LAMPORTS_FOR_TEST).unwrap();
    Env { svm, payer }
}

fn event_pda(organizer: &Pubkey, slug: &str) -> Pubkey {
    Pubkey::find_program_address(
        &[b"event", organizer.as_ref(), slug.as_bytes()],
        &PROGRAM_ID,
    )
    .0
}

fn vault_pda(event: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"vault", event.as_ref()], &PROGRAM_ID).0
}

fn now(svm: &LiteSVM) -> i64 {
    svm.get_sysvar::<Clock>().unix_timestamp
}

fn warp(svm: &mut LiteSVM, timestamp: i64) {
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = timestamp;
    svm.set_sysvar::<Clock>(&clock);
}

fn params(now_ts: i64) -> EventParams {
    EventParams {
        title: "Solana Break 2026".to_string(),
        description: "Community meetup".to_string(),
        venue: "Loft Hall".to_string(),
        city: "Almaty".to_string(),
        image_uri: "https://example.com/img.png".to_string(),
        starts_at: now_ts + 3600,
        ends_at: now_ts + 7200,
        ticket_price_lamports: 50_000_000,
        capacity: 100,
        hot_sale: false,
    }
}

fn sighash(name: &str) -> [u8; 8] {
    let preimage = format!("global:{}", name);
    let hash = Sha256::digest(preimage.as_bytes());
    let mut out = [0u8; 8];
    out.copy_from_slice(&hash[..8]);
    out
}

fn ix_data(ix_name: &str, args: &impl BorshSerialize) -> Vec<u8> {
    let mut data = sighash(ix_name).to_vec();
    args.serialize(&mut data).unwrap();
    data
}

fn init_ix(organizer: &Pubkey, slug: &str, params: &EventParams) -> Instruction {
    let event = event_pda(organizer, slug);
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*organizer, true),
            AccountMeta::new(event, false),
            AccountMeta::new(vault_pda(&event), false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: ix_data("init_event", &(slug.to_string(), params.clone())),
    }
}

fn sale_pda(event: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"sale", event.as_ref()], &PROGRAM_ID).0
}

fn queue_pda(event: &Pubkey, buyer: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"queue", event.as_ref(), buyer.as_ref()], &PROGRAM_ID).0
}

fn update_hot_ix(organizer: &Pubkey, slug: &str, params: &EventParams) -> Instruction {
    let mut ix = update_ix(organizer, slug, params);
    let event = event_pda(organizer, slug);
    ix.accounts[2] = AccountMeta::new_readonly(sale_pda(&event), false);
    ix
}

fn update_ix(organizer: &Pubkey, slug: &str, params: &EventParams) -> Instruction {
    let event = event_pda(organizer, slug);
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*organizer, true),
            AccountMeta::new(event, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ],
        data: ix_data("update_event", &(slug.to_string(), params.clone())),
    }
}

fn cancel_ix(organizer: &Pubkey, slug: &str) -> Instruction {
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*organizer, true),
            AccountMeta::new(event_pda(organizer, slug), false),
        ],
        data: ix_data("cancel_event", &(slug.to_string(),)),
    }
}

fn withdraw_hot_ix(organizer: &Pubkey, slug: &str, amount: u64) -> Instruction {
    let mut ix = withdraw_ix(organizer, slug, amount);
    let event = event_pda(organizer, slug);
    ix.accounts[2] = AccountMeta::new_readonly(sale_pda(&event), false);
    ix
}

fn withdraw_ix(organizer: &Pubkey, slug: &str, amount: u64) -> Instruction {
    let event = event_pda(organizer, slug);
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*organizer, true),
            AccountMeta::new_readonly(event, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
            AccountMeta::new(vault_pda(&event), false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: ix_data("withdraw", &(slug.to_string(), amount)),
    }
}

fn fund_ix(from: Pubkey, to: Pubkey, lamports: u64) -> Instruction {
    let mut data = Vec::with_capacity(12);
    data.extend_from_slice(&2u32.to_le_bytes());
    data.extend_from_slice(&lamports.to_le_bytes());
    Instruction {
        program_id: SYSTEM_PROGRAM_ID,
        accounts: vec![AccountMeta::new(from, true), AccountMeta::new(to, false)],
        data,
    }
}

fn send(env: &mut Env, ix: Instruction, extra_signers: &[&Keypair]) -> TransactionResult {
    let mut signers = vec![&env.payer];
    signers.extend_from_slice(extra_signers);
    let tx = Transaction::new(
        &signers,
        Message::new(&[ix, memo_ix()], Some(&env.payer.pubkey())),
        env.svm.latest_blockhash(),
    );
    env.svm.send_transaction(tx)
}

fn load_event(svm: &LiteSVM, pda: &Pubkey) -> Event {
    let account = svm.get_account(pda).unwrap();
    Event::deserialize(&mut &account.data[8..]).unwrap()
}

fn lamports(svm: &LiteSVM, key: &Pubkey) -> u64 {
    svm.get_account(key).unwrap().lamports
}

#[test]
fn init_event_creates_event_account() {
    let mut env = setup();
    let organizer = env.payer.pubkey();

    let p = params(now(&env.svm));
    send(&mut env, init_ix(&organizer, SLUG, &p), &[]).unwrap();

    let event = load_event(&env.svm, &event_pda(&organizer, SLUG));
    assert_eq!(event.organizer.as_ref(), organizer.as_ref());
    assert_eq!(event.slug, SLUG);
    assert_eq!(event.title, "Solana Break 2026");
    assert_eq!(event.city, "Almaty");
    assert_eq!(event.tickets_sold, 0);
    assert_eq!(event.capacity, 100);
    assert_eq!(event.status, EventStatus::Active);

    let vault = env
        .svm
        .get_account(&vault_pda(&event_pda(&organizer, SLUG)));
    assert!(vault.is_some());
    assert_eq!(vault.unwrap().data.len(), 0);
}

#[test]
fn init_event_rejects_duplicate_slug() {
    let mut env = setup();
    let organizer = env.payer.pubkey();

    let p = params(now(&env.svm));
    send(&mut env, init_ix(&organizer, SLUG, &p), &[]).unwrap();

    let res = send(&mut env, init_ix(&organizer, SLUG, &p), &[]);
    assert!(res.is_err());
}

#[test]
fn init_event_validates_inputs() {
    let mut env = setup();
    let organizer = env.payer.pubkey();

    let bad_slug = init_ix(&organizer, "Bad Slug!", &params(now(&env.svm)));
    assert!(send(&mut env, bad_slug, &[]).is_err());

    let mut zero_capacity = params(now(&env.svm));
    zero_capacity.capacity = 0;
    assert!(send(
        &mut env,
        init_ix(&organizer, "cap-zero", &zero_capacity),
        &[]
    )
    .is_err());

    let mut bad_dates = params(now(&env.svm));
    bad_dates.ends_at = bad_dates.starts_at;
    assert!(send(&mut env, init_ix(&organizer, "bad-dates", &bad_dates), &[]).is_err());
}

#[test]
fn update_event_changes_fields() {
    let mut env = setup();
    let organizer = env.payer.pubkey();

    let p = params(now(&env.svm));
    send(&mut env, init_ix(&organizer, SLUG, &p), &[]).unwrap();

    let mut updated = params(now(&env.svm));
    updated.title = "Solana Break 2026 v2".to_string();
    updated.ticket_price_lamports = 75_000_000;
    updated.capacity = 200;

    send(&mut env, update_ix(&organizer, SLUG, &updated), &[]).unwrap();

    let event = load_event(&env.svm, &event_pda(&organizer, SLUG));
    assert_eq!(event.title, "Solana Break 2026 v2");
    assert_eq!(event.ticket_price_lamports, 75_000_000);
    assert_eq!(event.capacity, 200);
}

#[test]
fn update_event_rejects_foreign_organizer() {
    let mut env = setup();
    let organizer = env.payer.pubkey();
    let stranger = Keypair::new();

    let p = params(now(&env.svm));
    send(&mut env, init_ix(&organizer, SLUG, &p), &[]).unwrap();

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(stranger.pubkey(), true),
            AccountMeta::new(event_pda(&organizer, SLUG), false),
        ],
        data: ix_data("update_event", &(SLUG.to_string(), params(now(&env.svm)))),
    };

    assert!(send(&mut env, ix, &[&stranger]).is_err());
}

#[test]
fn update_event_rejects_after_start() {
    let mut env = setup();
    let organizer = env.payer.pubkey();

    let init = params(now(&env.svm));
    send(&mut env, init_ix(&organizer, SLUG, &init), &[]).unwrap();

    warp(&mut env.svm, init.starts_at + 10);

    assert!(send(&mut env, update_ix(&organizer, SLUG, &init), &[]).is_err());
}

#[test]
fn cancel_event_marks_cancelled_and_rejects_twice() {
    let mut env = setup();
    let organizer = env.payer.pubkey();

    let p = params(now(&env.svm));
    send(&mut env, init_ix(&organizer, SLUG, &p), &[]).unwrap();

    send(&mut env, cancel_ix(&organizer, SLUG), &[]).unwrap();
    assert_eq!(
        load_event(&env.svm, &event_pda(&organizer, SLUG)).status,
        EventStatus::Cancelled
    );

    assert!(send(&mut env, cancel_ix(&organizer, SLUG), &[]).is_err());
}

#[test]
fn withdraw_fails_before_end() {
    let mut env = setup();
    let organizer = env.payer.pubkey();

    let p = params(now(&env.svm));
    send(&mut env, init_ix(&organizer, SLUG, &p), &[]).unwrap();

    let vault = vault_pda(&event_pda(&organizer, SLUG));
    send(&mut env, fund_ix(organizer, vault, 500_000_000), &[]).unwrap();

    assert!(send(&mut env, withdraw_ix(&organizer, SLUG, 100_000_000), &[]).is_err());
}

#[test]
fn withdraw_succeeds_after_end() {
    let mut env = setup();
    let organizer = env.payer.pubkey();
    let event = event_pda(&organizer, SLUG);

    let init = params(now(&env.svm));
    send(&mut env, init_ix(&organizer, SLUG, &init), &[]).unwrap();

    let vault = vault_pda(&event);
    send(&mut env, fund_ix(organizer, vault, 500_000_000), &[]).unwrap();

    let organizer_before = lamports(&env.svm, &organizer);
    let vault_before = lamports(&env.svm, &vault);

    warp(&mut env.svm, init.ends_at + 1);
    send(&mut env, withdraw_ix(&organizer, SLUG, 400_000_000), &[]).unwrap();

    let organizer_after = lamports(&env.svm, &organizer);
    let vault_after = lamports(&env.svm, &vault);

    assert_eq!(vault_before - vault_after, 400_000_000);
    let fee_allowance = 20_000;
    assert!(organizer_after > organizer_before + 400_000_000 - fee_allowance);
}

#[test]
fn withdraw_rejects_draining_rent_and_zero() {
    let mut env = setup();
    let organizer = env.payer.pubkey();
    let event = event_pda(&organizer, SLUG);

    let init = params(now(&env.svm));
    send(&mut env, init_ix(&organizer, SLUG, &init), &[]).unwrap();

    let vault = vault_pda(&event);
    send(&mut env, fund_ix(organizer, vault, 500_000_000), &[]).unwrap();

    warp(&mut env.svm, init.ends_at + 1);

    let full = lamports(&env.svm, &vault);
    assert!(send(&mut env, withdraw_ix(&organizer, SLUG, full), &[]).is_err());
    assert!(send(&mut env, withdraw_ix(&organizer, SLUG, 0), &[]).is_err());
}

const TOKEN22_PROGRAM_ID: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const ATA_PROGRAM_ID: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

fn mint_pda(event: &Pubkey, buyer: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"mint", event.as_ref(), buyer.as_ref()], &PROGRAM_ID).0
}

fn ticket_pda(event: &Pubkey, buyer: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"ticket", event.as_ref(), buyer.as_ref()], &PROGRAM_ID).0
}

fn buyer_ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[owner.as_ref(), TOKEN22_PROGRAM_ID.as_ref(), mint.as_ref()],
        &ATA_PROGRAM_ID,
    )
    .0
}

fn buy_ix(
    buyer: &Pubkey,
    event: &Pubkey,
    vault: &Pubkey,
    mint: &Pubkey,
    ticket: &Pubkey,
    ata: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*buyer, true),
            AccountMeta::new(*event, false),
            AccountMeta::new(*vault, false),
            AccountMeta::new(*mint, false),
            AccountMeta::new(*ticket, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
            AccountMeta::new(*ata, false),
            AccountMeta::new_readonly(TOKEN22_PROGRAM_ID, false),
            AccountMeta::new_readonly(ATA_PROGRAM_ID, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: ix_data("buy_ticket", &()),
    }
}

fn buy_hot_ix(
    buyer: &Pubkey,
    event: &Pubkey,
    vault: &Pubkey,
    mint: &Pubkey,
    ticket: &Pubkey,
    ata: &Pubkey,
) -> Instruction {
    let mut ix = buy_ix(buyer, event, vault, mint, ticket, ata);
    ix.accounts[5] = AccountMeta::new(sale_pda(event), false);
    ix.accounts[6] = AccountMeta::new(queue_pda(event, buyer), false);
    ix
}

fn check_in_ix(organizer: &Pubkey, event: &Pubkey, ticket: &Pubkey, slug: &str) -> Instruction {
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*organizer, true),
            AccountMeta::new(*event, false),
            AccountMeta::new(*ticket, false),
        ],
        data: ix_data("check_in", &(slug.to_string(),)),
    }
}

fn send_from(env: &mut Env, ix: Instruction, payer: &Keypair) -> TransactionResult {
    let tx = Transaction::new(
        &[payer],
        Message::new(&[ix, memo_ix()], Some(&payer.pubkey())),
        env.svm.latest_blockhash(),
    );
    env.svm.send_transaction(tx)
}

static TX_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn memo_ix() -> Instruction {
    let n = TX_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Instruction {
        program_id: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
        accounts: vec![],
        data: format!("tx-{}", n).into_bytes(),
    }
}

struct EventFixture {
    event: Pubkey,
    vault: Pubkey,
    params: EventParams,
}

fn create_event(env: &mut Env, slug: &str, mut params: EventParams) -> EventFixture {
    params.starts_at = now(&env.svm) + 3600;
    params.ends_at = params.starts_at + 3600;
    let organizer = env.payer.pubkey();
    send(env, init_ix(&organizer, slug, &params), &[]).unwrap();
    let event = event_pda(&organizer, slug);
    EventFixture {
        vault: vault_pda(&event),
        event,
        params,
    }
}

fn ata_amount(svm: &LiteSVM, ata: &Pubkey) -> u64 {
    let account = svm.get_account(ata).unwrap();
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&account.data[64..72]);
    u64::from_le_bytes(buf)
}

#[test]
fn buy_ticket_mints_nft_and_pays_vault() {
    let mut env = setup();
    let p = params(now(&env.svm));
    let fx = create_event(&mut env, "nft-show", p);
    let buyer = env.payer.pubkey();
    let mint = mint_pda(&fx.event, &buyer);
    let ticket = ticket_pda(&fx.event, &buyer);
    let ata = buyer_ata(&buyer, &mint);

    let vault_before = lamports(&env.svm, &fx.vault);
    send(
        &mut env,
        buy_ix(&buyer, &fx.event, &fx.vault, &mint, &ticket, &ata),
        &[],
    )
    .unwrap();

    let t = load_event(&env.svm, &fx.event);
    assert_eq!(t.tickets_sold, 1);
    assert_eq!(
        lamports(&env.svm, &fx.vault),
        vault_before + fx.params.ticket_price_lamports
    );
    assert_eq!(ata_amount(&env.svm, &ata), 1);

    let ticket_acc = env.svm.get_account(&ticket).unwrap();
    let ticket_data = events::state::Ticket::deserialize(&mut &ticket_acc.data[8..]).unwrap();
    assert_eq!(ticket_data.status, TicketStatus::Valid);
    assert_eq!(ticket_data.mint, mint);
    assert_eq!(ticket_data.buyer.as_ref(), buyer.as_ref());

    use spl_token_2022_interface::extension::BaseStateWithExtensions;
    use spl_token_2022_interface::extension::StateWithExtensions;
    use spl_token_2022_interface::state::Mint as SplMint;
    use spl_token_metadata_interface::state::TokenMetadata;

    let mint_acc = env.svm.get_account(&mint).unwrap();
    let mint_state = StateWithExtensions::<SplMint>::unpack(&mint_acc.data).unwrap();
    let metadata = mint_state
        .get_variable_len_extension::<TokenMetadata>()
        .unwrap();
    assert_eq!(metadata.name, "Solana Break 2026 #1");
    assert_eq!(metadata.symbol, "AFISHA");
    assert_eq!(metadata.uri, "https://example.com/img.png");
    assert_eq!(mint_state.base.supply, 1);
}

#[test]
fn buy_ticket_rejects_second_for_same_wallet() {
    let mut env = setup();
    let p = params(now(&env.svm));
    let fx = create_event(&mut env, "one-per-wallet", p);
    let buyer = env.payer.pubkey();
    let mint = mint_pda(&fx.event, &buyer);
    let ticket = ticket_pda(&fx.event, &buyer);
    let ata = buyer_ata(&buyer, &mint);

    let ix = buy_ix(&buyer, &fx.event, &fx.vault, &mint, &ticket, &ata);
    send(&mut env, ix.clone(), &[]).unwrap();
    assert!(send(&mut env, ix, &[]).is_err());
}

#[test]
fn buy_ticket_rejects_sold_out() {
    let mut env = setup();
    let mut p = params(now(&env.svm));
    p.capacity = 1;
    let fx = create_event(&mut env, "tiny-venue", p.clone());
    let organizer = env.payer.pubkey();
    let buyer2 = Keypair::new();
    env.svm.airdrop(&buyer2.pubkey(), 10_000_000_000).unwrap();

    let mint1 = mint_pda(&fx.event, &organizer);
    let ata1 = buyer_ata(&organizer, &mint1);
    send(
        &mut env,
        buy_ix(
            &organizer,
            &fx.event,
            &fx.vault,
            &mint1,
            &ticket_pda(&fx.event, &organizer),
            &ata1,
        ),
        &[],
    )
    .unwrap();

    let mint2 = mint_pda(&fx.event, &buyer2.pubkey());
    let ata2 = buyer_ata(&buyer2.pubkey(), &mint2);
    let ix = buy_ix(
        &buyer2.pubkey(),
        &fx.event,
        &fx.vault,
        &mint2,
        &ticket_pda(&fx.event, &buyer2.pubkey()),
        &ata2,
    );
    assert!(send_from(&mut env, ix, &buyer2).is_err());
}

#[test]
fn buy_ticket_rejects_after_start_and_cancelled() {
    let mut env = setup();
    let p = params(now(&env.svm));
    let fx = create_event(&mut env, "late-show", p);
    let buyer = env.payer.pubkey();
    let mint = mint_pda(&fx.event, &buyer);
    let ticket = ticket_pda(&fx.event, &buyer);
    let ata = buyer_ata(&buyer, &mint);
    let ix = buy_ix(&buyer, &fx.event, &fx.vault, &mint, &ticket, &ata);

    warp(&mut env.svm, fx.params.starts_at + 1);
    assert!(send(&mut env, ix.clone(), &[]).is_err());
    warp(&mut env.svm, fx.params.starts_at - 100);

    send(&mut env, cancel_ix(&buyer, "late-show"), &[]).unwrap();
    assert!(send(&mut env, ix, &[]).is_err());
}

#[test]
fn buy_ticket_free_event_works() {
    let mut env = setup();
    let mut p = params(now(&env.svm));
    p.ticket_price_lamports = 0;
    let fx = create_event(&mut env, "free-entry", p);
    let buyer = env.payer.pubkey();
    let mint = mint_pda(&fx.event, &buyer);
    let ticket = ticket_pda(&fx.event, &buyer);
    let ata = buyer_ata(&buyer, &mint);

    let vault_before = lamports(&env.svm, &fx.vault);
    send(
        &mut env,
        buy_ix(&buyer, &fx.event, &fx.vault, &mint, &ticket, &ata),
        &[],
    )
    .unwrap();
    assert_eq!(lamports(&env.svm, &fx.vault), vault_before);
    assert_eq!(ata_amount(&env.svm, &ata), 1);
}

#[test]
fn check_in_marks_used_and_rejects_twice() {
    let mut env = setup();
    let p = params(now(&env.svm));
    let fx = create_event(&mut env, "door-scan", p);
    let organizer = env.payer.pubkey();
    let mint = mint_pda(&fx.event, &organizer);
    let ticket = ticket_pda(&fx.event, &organizer);
    let ata = buyer_ata(&organizer, &mint);

    send(
        &mut env,
        buy_ix(&organizer, &fx.event, &fx.vault, &mint, &ticket, &ata),
        &[],
    )
    .unwrap();

    warp(&mut env.svm, fx.params.starts_at);

    let ix = check_in_ix(&organizer, &fx.event, &ticket, "door-scan");
    send(&mut env, ix.clone(), &[]).unwrap();
    assert!(send(&mut env, ix, &[]).is_err());

    let ticket_acc = env.svm.get_account(&ticket).unwrap();
    let ticket_data = events::state::Ticket::deserialize(&mut &ticket_acc.data[8..]).unwrap();
    assert_eq!(ticket_data.status, TicketStatus::Used);
    assert!(ticket_data.checked_in_at > 0);
}

#[test]
fn check_in_rejects_foreign_organizer() {
    let mut env = setup();
    let p = params(now(&env.svm));
    let fx = create_event(&mut env, "strict-door", p);
    let organizer = env.payer.pubkey();
    let mint = mint_pda(&fx.event, &organizer);
    let ticket = ticket_pda(&fx.event, &organizer);
    let ata = buyer_ata(&organizer, &mint);

    send(
        &mut env,
        buy_ix(&organizer, &fx.event, &fx.vault, &mint, &ticket, &ata),
        &[],
    )
    .unwrap();
    warp(&mut env.svm, fx.params.starts_at);

    let stranger = Keypair::new();
    let ix = check_in_ix(&stranger.pubkey(), &fx.event, &ticket, "strict-door");
    assert!(send_from(&mut env, ix, &stranger).is_err());
}

#[test]
fn check_in_rejects_too_early_and_too_late() {
    let mut env = setup();
    let p = params(now(&env.svm));
    let fx = create_event(&mut env, "time-window", p);
    let organizer = env.payer.pubkey();
    let mint = mint_pda(&fx.event, &organizer);
    let ticket = ticket_pda(&fx.event, &organizer);
    let ata = buyer_ata(&organizer, &mint);

    send(
        &mut env,
        buy_ix(&organizer, &fx.event, &fx.vault, &mint, &ticket, &ata),
        &[],
    )
    .unwrap();

    warp(&mut env.svm, fx.params.starts_at - 4000);
    let ix = check_in_ix(&organizer, &fx.event, &ticket, "time-window");
    assert!(send(&mut env, ix.clone(), &[]).is_err());

    warp(&mut env.svm, fx.params.ends_at + 25 * 3600);
    assert!(send(&mut env, ix, &[]).is_err());
}

const SLOT_HASHES_SYSVAR: Pubkey = pubkey!("SysvarS1otHashes111111111111111111111111111");
const STAKE: u64 = 50_000_000;

use events::state::SaleParams;

fn configure_sale_ix(organizer: &Pubkey, slug: &str, params: &SaleParams) -> Instruction {
    let event = event_pda(organizer, slug);
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*organizer, true),
            AccountMeta::new(event, false),
            AccountMeta::new(sale_pda(&event), false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: ix_data("configure_sale", &(slug.to_string(), params.clone())),
    }
}

fn join_queue_ix(buyer: &Pubkey, event: &Pubkey) -> Instruction {
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*buyer, true),
            AccountMeta::new_readonly(*event, false),
            AccountMeta::new(sale_pda(event), false),
            AccountMeta::new(queue_pda(event, buyer), false),
            AccountMeta::new(vault_pda(event), false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: ix_data("join_queue", &()),
    }
}

fn settle_randomness_ix(caller: &Pubkey, event: &Pubkey) -> Instruction {
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*caller, true),
            AccountMeta::new_readonly(*event, false),
            AccountMeta::new(sale_pda(event), false),
            AccountMeta::new_readonly(SLOT_HASHES_SYSVAR, false),
        ],
        data: ix_data("settle_randomness", &()),
    }
}

fn settle_stake_ix(caller: &Pubkey, event: &Pubkey, entry_buyer: &Pubkey) -> Instruction {
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*caller, true),
            AccountMeta::new_readonly(*event, false),
            AccountMeta::new(sale_pda(event), false),
            AccountMeta::new(queue_pda(event, entry_buyer), false),
            AccountMeta::new(*entry_buyer, false),
            AccountMeta::new(vault_pda(event), false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: ix_data("settle_stake", &()),
    }
}

struct HotFixture {
    event: Pubkey,
    vault: Pubkey,
    sale: Pubkey,
    claim_start: i64,
    round: i64,
    starts_at: i64,
    ends_at: i64,
}

fn hot_params(now_ts: i64) -> events::state::EventParams {
    let mut p = params(now_ts);
    p.hot_sale = true;
    p.ticket_price_lamports = 150_000_000;
    p.starts_at = now_ts + 200_000;
    p.ends_at = now_ts + 206_400;
    p
}

fn sale_params(now_ts: i64, window: u32) -> SaleParams {
    SaleParams {
        registration_start: now_ts + 60,
        registration_end: now_ts + 600,
        reveal_at: now_ts + 700,
        claim_start: now_ts + 800,
        round_duration_secs: 300,
        stake_lamports: STAKE,
        window_size: window,
    }
}

fn create_hot_event(env: &mut Env, slug: &str, capacity: u32, window: u32) -> HotFixture {
    let t0 = now(&env.svm);
    let mut p = hot_params(t0);
    p.capacity = capacity;
    let organizer = env.payer.pubkey();
    send(env, init_ix(&organizer, slug, &p), &[]).unwrap();

    let sp = sale_params(t0, window);
    send(env, configure_sale_ix(&organizer, slug, &sp), &[]).unwrap();

    warp(&mut env.svm, t0 + 100);
    let event = event_pda(&organizer, slug);
    HotFixture {
        vault: vault_pda(&event),
        sale: sale_pda(&event),
        event,
        claim_start: sp.claim_start,
        round: sp.round_duration_secs,
        starts_at: p.starts_at,
        ends_at: p.ends_at,
    }
}

fn load_sale(svm: &LiteSVM, pda: &Pubkey) -> events::state::SaleState {
    let account = svm.get_account(pda).unwrap();
    events::state::SaleState::deserialize(&mut &account.data[8..]).unwrap()
}

fn load_entry(svm: &LiteSVM, pda: &Pubkey) -> events::state::QueueEntry {
    let account = svm.get_account(pda).unwrap();
    events::state::QueueEntry::deserialize(&mut &account.data[8..]).unwrap()
}

fn settle_and_warp_to_round(env: &mut Env, fx: &HotFixture, entry_buyer: &Pubkey) {
    let caller = env.payer.pubkey();
    warp(&mut env.svm, fx.claim_start - 100);
    send(env, settle_randomness_ix(&caller, &fx.event), &[]).unwrap();

    let sale = load_sale(&env.svm, &fx.sale);
    let entry = load_entry(&env.svm, &queue_pda(&fx.event, entry_buyer));
    let eff = ((sale.randomness % sale.total_entries as u64)
        + (entry.position as u64 % sale.total_entries as u64)) as u32;
    let round = eff as i64 / sale.window_size as i64;
    warp(&mut env.svm, fx.claim_start + round * fx.round + 5);
}

#[test]
fn configure_sale_creates_state() {
    let mut env = setup();
    let fx = create_hot_event(&mut env, "hot-concert", 500, 10);

    let sale = load_sale(&env.svm, &fx.sale);
    assert_eq!(sale.event, fx.event);
    assert_eq!(sale.stake_lamports, STAKE);
    assert_eq!(sale.window_size, 10);
    assert_eq!(sale.total_entries, 0);
    assert!(!sale.settled);
    assert_eq!(sale.pending(), 0);
}

#[test]
fn configure_sale_rejects_non_hot_event() {
    let mut env = setup();
    let p = params(now(&env.svm));
    let fx = create_event(&mut env, "cold-event", p);
    let organizer = env.payer.pubkey();
    let t0 = now(&env.svm);
    let ix = configure_sale_ix(&organizer, "cold-event", &sale_params(t0, 10));
    assert!(send(&mut env, ix, &[]).is_err());
    assert!(env.svm.get_account(&sale_pda(&fx.event)).is_none());
}

#[test]
fn configure_sale_rejects_bad_phases() {
    let mut env = setup();
    let organizer = env.payer.pubkey();
    let t0 = now(&env.svm);
    send(
        &mut env,
        init_ix(&organizer, "bad-phases", &hot_params(t0)),
        &[],
    )
    .unwrap();

    let mut sp = sale_params(t0, 10);
    sp.claim_start = sp.reveal_at - 100;
    assert!(send(
        &mut env,
        configure_sale_ix(&organizer, "bad-phases", &sp),
        &[]
    )
    .is_err());
}

#[test]
fn join_queue_stakes_and_assigns_positions() {
    let mut env = setup();
    let fx = create_hot_event(&mut env, "queue-assign", 500, 10);
    let buyer1 = env.payer.pubkey();
    let buyer2 = Keypair::new();
    env.svm.airdrop(&buyer2.pubkey(), 10_000_000_000).unwrap();

    let vault_before = lamports(&env.svm, &fx.vault);
    send(&mut env, join_queue_ix(&buyer1, &fx.event), &[]).unwrap();
    send_from(
        &mut env,
        join_queue_ix(&buyer2.pubkey(), &fx.event),
        &buyer2,
    )
    .unwrap();

    assert_eq!(lamports(&env.svm, &fx.vault), vault_before + 2 * STAKE);
    let sale = load_sale(&env.svm, &fx.sale);
    assert_eq!(sale.total_entries, 2);
    assert_eq!(sale.pending(), 2);

    let e1 = load_entry(&env.svm, &queue_pda(&fx.event, &buyer1));
    let e2 = load_entry(&env.svm, &queue_pda(&fx.event, &buyer2.pubkey()));
    assert_eq!((e1.position, e2.position), (0, 1));
    assert_eq!(e1.status, events::state::QueueStatus::Staked);
}

#[test]
fn join_queue_rejects_outside_window_and_twice() {
    let mut env = setup();
    let fx = create_hot_event(&mut env, "queue-window", 500, 10);
    let buyer = env.payer.pubkey();

    warp(&mut env.svm, fx.claim_start - 5);
    assert!(send(&mut env, join_queue_ix(&buyer, &fx.event), &[]).is_err());

    warp(&mut env.svm, fx.claim_start - 600);
    send(&mut env, join_queue_ix(&buyer, &fx.event), &[]).unwrap();
    let dup = join_queue_ix(&buyer, &fx.event);
    assert!(send(&mut env, dup, &[]).is_err());
}

#[test]
fn settle_randomness_phases() {
    let mut env = setup();
    let fx = create_hot_event(&mut env, "reveal-flow", 500, 10);
    let buyer = env.payer.pubkey();
    warp(&mut env.svm, fx.claim_start - 300);
    send(&mut env, join_queue_ix(&buyer, &fx.event), &[]).unwrap();

    let caller = env.payer.pubkey();
    warp(&mut env.svm, fx.claim_start - 150);
    assert!(send(&mut env, settle_randomness_ix(&caller, &fx.event), &[]).is_err());

    warp(&mut env.svm, fx.claim_start - 50);
    send(&mut env, settle_randomness_ix(&caller, &fx.event), &[]).unwrap();
    let sale = load_sale(&env.svm, &fx.sale);
    assert!(sale.settled);

    let dup = settle_randomness_ix(&caller, &fx.event);
    assert!(send(&mut env, dup, &[]).is_err());
}

#[test]
fn queue_buy_flow_mints_ticket_and_claims_stake() {
    let mut env = setup();
    let fx = create_hot_event(&mut env, "claim-flow", 500, 10);
    let buyer = env.payer.pubkey();
    warp(&mut env.svm, fx.claim_start - 300);
    send(&mut env, join_queue_ix(&buyer, &fx.event), &[]).unwrap();

    settle_and_warp_to_round(&mut env, &fx, &buyer);

    let mint = mint_pda(&fx.event, &buyer);
    let ticket = ticket_pda(&fx.event, &buyer);
    let ata = buyer_ata(&buyer, &mint);

    let vault_before = lamports(&env.svm, &fx.vault);
    send(
        &mut env,
        buy_hot_ix(&buyer, &fx.event, &fx.vault, &mint, &ticket, &ata),
        &[],
    )
    .unwrap();

    let full_price = load_event(&env.svm, &fx.event).ticket_price_lamports;
    assert_eq!(
        lamports(&env.svm, &fx.vault),
        vault_before + full_price - STAKE
    );

    let entry = load_entry(&env.svm, &queue_pda(&fx.event, &buyer));
    assert_eq!(entry.status, events::state::QueueStatus::Claimed);
    let sale = load_sale(&env.svm, &fx.sale);
    assert_eq!(sale.claimed, 1);
    assert_eq!(sale.pending(), 0);
    assert_eq!(ata_amount(&env.svm, &ata), 1);
}

#[test]
fn queue_buy_rejects_wrong_round_and_early() {
    let mut env = setup();
    let fx = create_hot_event(&mut env, "round-gate", 500, 1);
    let buyer = env.payer.pubkey();
    warp(&mut env.svm, fx.claim_start - 300);
    send(&mut env, join_queue_ix(&buyer, &fx.event), &[]).unwrap();

    let caller = env.payer.pubkey();
    warp(&mut env.svm, fx.claim_start - 50);
    send(&mut env, settle_randomness_ix(&caller, &fx.event), &[]).unwrap();

    let mint = mint_pda(&fx.event, &buyer);
    let ticket = ticket_pda(&fx.event, &buyer);
    let ata = buyer_ata(&buyer, &mint);
    let ix = buy_hot_ix(&buyer, &fx.event, &fx.vault, &mint, &ticket, &ata);

    warp(&mut env.svm, fx.claim_start + 999 * fx.round + 5);
    assert!(send(&mut env, ix.clone(), &[]).is_err());

    let sale = load_sale(&env.svm, &fx.sale);
    let entry = load_entry(&env.svm, &queue_pda(&fx.event, &buyer));
    let eff = ((sale.randomness % sale.total_entries as u64)
        + (entry.position as u64 % sale.total_entries as u64)) as u32;
    let round = eff as i64 / sale.window_size as i64;
    warp(&mut env.svm, fx.claim_start + round * fx.round + 5);
    let fresh = buy_hot_ix(&buyer, &fx.event, &fx.vault, &mint, &ticket, &ata);
    send(&mut env, fresh, &[]).unwrap();
}

#[test]
fn buy_before_settlement_fails() {
    let mut env = setup();
    let fx = create_hot_event(&mut env, "no-settle", 500, 10);
    let buyer = env.payer.pubkey();
    warp(&mut env.svm, fx.claim_start - 300);
    send(&mut env, join_queue_ix(&buyer, &fx.event), &[]).unwrap();

    warp(&mut env.svm, fx.claim_start + 5);
    let mint = mint_pda(&fx.event, &buyer);
    let ticket = ticket_pda(&fx.event, &buyer);
    let ata = buyer_ata(&buyer, &mint);
    let ix = buy_hot_ix(&buyer, &fx.event, &fx.vault, &mint, &ticket, &ata);
    assert!(send(&mut env, ix, &[]).is_err());
}

#[test]
fn no_show_forfeits_stake() {
    let mut env = setup();
    let fx = create_hot_event(&mut env, "no-show", 500, 1);
    let buyer = env.payer.pubkey();
    warp(&mut env.svm, fx.claim_start - 300);
    send(&mut env, join_queue_ix(&buyer, &fx.event), &[]).unwrap();

    let caller = env.payer.pubkey();
    warp(&mut env.svm, fx.claim_start - 50);
    send(&mut env, settle_randomness_ix(&caller, &fx.event), &[]).unwrap();

    warp(&mut env.svm, fx.starts_at + 10);

    let vault_before = lamports(&env.svm, &fx.vault);
    let buyer_before = lamports(&env.svm, &buyer);
    send(&mut env, settle_stake_ix(&caller, &fx.event, &buyer), &[]).unwrap();

    let entry = load_entry(&env.svm, &queue_pda(&fx.event, &buyer));
    assert_eq!(entry.status, events::state::QueueStatus::Forfeited);
    assert_eq!(lamports(&env.svm, &fx.vault), vault_before);
    let fee = 10_000;
    assert!(lamports(&env.svm, &buyer) < buyer_before + fee);

    warp(&mut env.svm, fx.ends_at + 1);
    let organizer = env.payer.pubkey();
    send(&mut env, withdraw_hot_ix(&organizer, "no-show", STAKE), &[]).unwrap();
}

#[test]
fn sold_out_losers_get_refund() {
    let mut env = setup();
    let fx = create_hot_event(&mut env, "sold-out", 1, 1);
    let winner_or_loser1 = env.payer.pubkey();
    let other = Keypair::new();
    env.svm.airdrop(&other.pubkey(), 10_000_000_000).unwrap();

    warp(&mut env.svm, fx.claim_start - 300);
    send(&mut env, join_queue_ix(&winner_or_loser1, &fx.event), &[]).unwrap();
    send_from(&mut env, join_queue_ix(&other.pubkey(), &fx.event), &other).unwrap();

    let caller = env.payer.pubkey();
    warp(&mut env.svm, fx.claim_start - 50);
    send(&mut env, settle_randomness_ix(&caller, &fx.event), &[]).unwrap();

    let sale = load_sale(&env.svm, &fx.sale);
    let e1 = load_entry(&env.svm, &queue_pda(&fx.event, &winner_or_loser1));
    let e2 = load_entry(&env.svm, &queue_pda(&fx.event, &other.pubkey()));
    let eff = |e: &events::state::QueueEntry| {
        ((sale.randomness % sale.total_entries as u64)
            + (e.position as u64 % sale.total_entries as u64)) as u32
    };
    let (first, second) = if eff(&e1) < eff(&e2) {
        (winner_or_loser1, other.pubkey())
    } else {
        (other.pubkey(), winner_or_loser1)
    };

    let round0 = (eff(&if eff(&e1) < eff(&e2) { e1 } else { e2 }) as i64) / 1;
    warp(&mut env.svm, fx.claim_start + round0 * fx.round + 5);

    let mint = mint_pda(&fx.event, &first);
    let ticket = ticket_pda(&fx.event, &first);
    let ata = buyer_ata(&first, &mint);
    if first == winner_or_loser1 {
        send(
            &mut env,
            buy_hot_ix(&first, &fx.event, &fx.vault, &mint, &ticket, &ata),
            &[],
        )
        .unwrap();
    } else {
        send_from(
            &mut env,
            buy_hot_ix(&first, &fx.event, &fx.vault, &mint, &ticket, &ata),
            &other,
        )
        .unwrap();
    }

    assert_eq!(load_event(&env.svm, &fx.event).tickets_sold, 1);

    let vault_before = lamports(&env.svm, &fx.vault);
    let loser_before = lamports(&env.svm, &second);
    let settle = settle_stake_ix(&second, &fx.event, &second);
    if second == winner_or_loser1 {
        send(&mut env, settle, &[]).unwrap();
    } else {
        send_from(&mut env, settle, &other).unwrap();
    }

    let entry = load_entry(&env.svm, &queue_pda(&fx.event, &second));
    assert_eq!(entry.status, events::state::QueueStatus::Settled);
    assert_eq!(lamports(&env.svm, &fx.vault), vault_before - STAKE);
    let fee = 10_000;
    assert!(lamports(&env.svm, &second) > loser_before + STAKE - fee);
}

#[test]
fn withdraw_blocked_until_stakes_settled() {
    let mut env = setup();
    let fx = create_hot_event(&mut env, "pending-guard", 500, 1);
    let buyer = env.payer.pubkey();
    warp(&mut env.svm, fx.claim_start - 300);
    send(&mut env, join_queue_ix(&buyer, &fx.event), &[]).unwrap();

    warp(&mut env.svm, fx.ends_at + 1);
    let organizer = env.payer.pubkey();
    assert!(send(
        &mut env,
        withdraw_hot_ix(&organizer, "pending-guard", STAKE),
        &[]
    )
    .is_err());

    warp(&mut env.svm, fx.starts_at + 10);
    send(
        &mut env,
        settle_stake_ix(&organizer, &fx.event, &buyer),
        &[],
    )
    .unwrap();

    warp(&mut env.svm, fx.ends_at + 1);
    send(
        &mut env,
        withdraw_hot_ix(&organizer, "pending-guard", STAKE),
        &[],
    )
    .unwrap();
}

#[test]
fn update_event_locks_sale_params_for_hot_events() {
    let mut env = setup();
    let fx = create_hot_event(&mut env, "locked-params", 500, 10);
    let organizer = env.payer.pubkey();

    let t0 = now(&env.svm);
    let mut changed_price = hot_params(t0);
    changed_price.ticket_price_lamports = 999_000_000;
    changed_price.capacity = 500;
    changed_price.starts_at = fx.starts_at;
    changed_price.ends_at = fx.ends_at;
    assert!(send(
        &mut env,
        update_hot_ix(&organizer, "locked-params", &changed_price),
        &[]
    )
    .is_err());

    let mut renamed = hot_params(t0);
    renamed.capacity = 500;
    renamed.starts_at = fx.starts_at;
    renamed.ends_at = fx.ends_at;
    renamed.title = "New title".to_string();
    send(
        &mut env,
        update_hot_ix(&organizer, "locked-params", &renamed),
        &[],
    )
    .unwrap();
}
