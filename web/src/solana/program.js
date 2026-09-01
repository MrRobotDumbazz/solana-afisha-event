import { PublicKey, TransactionInstruction } from '@solana/web3.js'
import { sha256 } from '@noble/hashes/sha256'
import { BorshWriter } from './borsh'

export const PROGRAM_ID = new PublicKey('7J6VC2HsTxBCBMc94FbcfT2NcN2bmSK5nhjejeuL4g8Y')
export const SYSTEM_PROGRAM_ID = new PublicKey('11111111111111111111111111111111')
export const TOKEN_2022_PROGRAM_ID = new PublicKey('TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb')
export const ATA_PROGRAM_ID = new PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL')

function disc(name) {
  return new Uint8Array(sha256(new TextEncoder().encode(`global:${name}`)).slice(0, 8))
}

export function pdaEvent(organizer, slug) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('event'), organizer.toBuffer(), Buffer.from(slug)],
    PROGRAM_ID,
  )[0]
}

export function pdaVault(event) {
  return PublicKey.findProgramAddressSync([Buffer.from('vault'), event.toBuffer()], PROGRAM_ID)[0]
}

export function pdaMint(event, buyer) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('mint'), event.toBuffer(), buyer.toBuffer()],
    PROGRAM_ID,
  )[0]
}

export function pdaTicket(event, buyer) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('ticket'), event.toBuffer(), buyer.toBuffer()],
    PROGRAM_ID,
  )[0]
}

export function pdaSale(event) {
  return PublicKey.findProgramAddressSync([Buffer.from('sale'), event.toBuffer()], PROGRAM_ID)[0]
}

export function pdaQueueEntry(event, buyer) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('queue'), event.toBuffer(), buyer.toBuffer()],
    PROGRAM_ID,
  )[0]
}

export function buyerAta(owner, mint) {
  return PublicKey.findProgramAddressSync(
    [owner.toBuffer(), TOKEN_2022_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    ATA_PROGRAM_ID,
  )[0]
}

function serializeEventParams(w, p) {
  w.str(p.title)
    .str(p.description)
    .str(p.venue)
    .str(p.city)
    .str(p.image_uri)
    .i64(p.starts_at)
    .i64(p.ends_at)
    .u64(p.ticket_price_lamports)
    .u32(p.capacity)
    .bool(p.hot_sale)
}

export function ixInitEvent(organizer, slug, params) {
  const w = new BorshWriter()
  w.str(slug)
  serializeEventParams(w, params)
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: organizer, isSigner: true, isWritable: true },
      { pubkey: pdaEvent(organizer, slug), isSigner: false, isWritable: true },
      { pubkey: pdaVault(pdaEvent(organizer, slug)), isSigner: false, isWritable: true },
      { pubkey: SYSTEM_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.concat([Buffer.from(disc('init_event')), w.toBuffer()]),
  })
}

export function ixJoinQueue(buyer, event) {
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: buyer, isSigner: true, isWritable: true },
      { pubkey: event, isSigner: false, isWritable: false },
      { pubkey: pdaSale(event), isSigner: false, isWritable: true },
      { pubkey: pdaQueueEntry(event, buyer), isSigner: false, isWritable: true },
      { pubkey: pdaVault(event), isSigner: false, isWritable: true },
      { pubkey: SYSTEM_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(disc('join_queue')),
  })
}

export function ixBuyTicket(buyer, event, { hot = false } = {}) {
  const mint = pdaMint(event, buyer)
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: buyer, isSigner: true, isWritable: true },
      { pubkey: event, isSigner: false, isWritable: true },
      { pubkey: pdaVault(event), isSigner: false, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: true },
      { pubkey: pdaTicket(event, buyer), isSigner: false, isWritable: true },
      { pubkey: hot ? pdaSale(event) : PROGRAM_ID, isSigner: false, isWritable: hot },
      {
        pubkey: hot ? pdaQueueEntry(event, buyer) : PROGRAM_ID,
        isSigner: false,
        isWritable: hot,
      },
      { pubkey: buyerAta(buyer, mint), isSigner: false, isWritable: true },
      { pubkey: TOKEN_2022_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: ATA_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SYSTEM_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(disc('buy_ticket')),
  })
}

export function ixCheckIn(organizer, event, ticket, slug) {
  const w = new BorshWriter()
  w.str(slug)
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: organizer, isSigner: true, isWritable: false },
      { pubkey: event, isSigner: false, isWritable: false },
      { pubkey: ticket, isSigner: false, isWritable: true },
    ],
    data: Buffer.concat([Buffer.from(disc('check_in')), w.toBuffer()]),
  })
}
