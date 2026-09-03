import { PublicKey, TransactionInstruction } from '@solana/web3.js'
import { sha256 } from '@noble/hashes/sha256'
import { BorshWriter } from './borsh'

export const PROGRAM_ID = new PublicKey('7J6VC2HsTxBCBMc94FbcfT2NcN2bmSK5nhjejeuL4g8Y')
export const SYSTEM_PROGRAM_ID = new PublicKey('11111111111111111111111111111111')
export const TOKEN_2022_PROGRAM_ID = new PublicKey('TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb')
export const ATA_PROGRAM_ID = new PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL')
export const SLOT_HASHES_SYSVAR = new PublicKey('SysvarS1otHashes111111111111111111111111111')

function disc(name: string): Uint8Array {
  return new Uint8Array(sha256(new TextEncoder().encode(`global:${name}`)).slice(0, 8))
}

export interface EventParams {
  title: string
  description: string
  venue: string
  city: string
  image_uri: string
  starts_at: bigint
  ends_at: bigint
  ticket_price_lamports: bigint
  capacity: number
  hot_sale: boolean
}

export interface SaleParams {
  registration_start: bigint
  registration_end: bigint
  reveal_at: bigint
  claim_start: bigint
  round_duration_secs: bigint
  stake_lamports: bigint
  window_size: number
}

export function pdaEvent(organizer: PublicKey, slug: string): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('event'), organizer.toBuffer(), Buffer.from(slug)],
    PROGRAM_ID,
  )[0]
}

export function pdaVault(event: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from('vault'), event.toBuffer()], PROGRAM_ID)[0]
}

export function pdaMint(event: PublicKey, buyer: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('mint'), event.toBuffer(), buyer.toBuffer()],
    PROGRAM_ID,
  )[0]
}

export function pdaTicket(event: PublicKey, buyer: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('ticket'), event.toBuffer(), buyer.toBuffer()],
    PROGRAM_ID,
  )[0]
}

export function pdaSale(event: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from('sale'), event.toBuffer()], PROGRAM_ID)[0]
}

export function pdaQueueEntry(event: PublicKey, buyer: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('queue'), event.toBuffer(), buyer.toBuffer()],
    PROGRAM_ID,
  )[0]
}

export function buyerAta(owner: PublicKey, mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [owner.toBuffer(), TOKEN_2022_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    ATA_PROGRAM_ID,
  )[0]
}

export function ixInitEvent(organizer: PublicKey, slug: string, params: EventParams) {
  const w = new BorshWriter()
  w.str(slug)
  w.str(params.title)
    .str(params.description)
    .str(params.venue)
    .str(params.city)
    .str(params.image_uri)
    .i64(params.starts_at)
    .i64(params.ends_at)
    .u64(params.ticket_price_lamports)
    .u32(params.capacity)
    .bool(params.hot_sale)
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

export function ixJoinQueue(buyer: PublicKey, event: PublicKey) {
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

export function ixConfigureSale(
  organizer: PublicKey,
  event: PublicKey,
  slug: string,
  params: SaleParams,
) {
  const w = new BorshWriter()
  w.str(slug)
  w.i64(params.registration_start)
    .i64(params.registration_end)
    .i64(params.reveal_at)
    .i64(params.claim_start)
    .i64(params.round_duration_secs)
    .u64(params.stake_lamports)
    .u32(params.window_size)
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: organizer, isSigner: true, isWritable: true },
      { pubkey: event, isSigner: false, isWritable: false },
      { pubkey: pdaSale(event), isSigner: false, isWritable: true },
      { pubkey: SYSTEM_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.concat([Buffer.from(disc('configure_sale')), w.toBuffer()]),
  })
}

export function ixSettleRandomness(caller: PublicKey, event: PublicKey) {
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: caller, isSigner: true, isWritable: false },
      { pubkey: event, isSigner: false, isWritable: false },
      { pubkey: pdaSale(event), isSigner: false, isWritable: true },
      { pubkey: SLOT_HASHES_SYSVAR, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(disc('settle_randomness')),
  })
}

export function ixBuyTicket(
  buyer: PublicKey,
  event: PublicKey,
  { hot = false }: { hot?: boolean } = {},
) {
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

export function ixCheckIn(
  organizer: PublicKey,
  event: PublicKey,
  ticket: PublicKey,
  slug: string,
) {
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
