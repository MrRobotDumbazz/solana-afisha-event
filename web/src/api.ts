import { Connection, clusterApiUrl } from '@solana/web3.js'

export const connection = new Connection(
  import.meta.env.VITE_RPC_URL || clusterApiUrl('devnet'),
  'confirmed',
)

export interface EventSummary {
  pubkey: string
  organizer: string
  slug: string
  title: string
  description: string
  venue: string
  city: string
  image_uri: string
  starts_at: number
  ends_at: number
  ticket_price_lamports: number
  capacity: number
  tickets_sold: number
  hot_sale: boolean
  sale_configured: boolean
  status: 'active' | 'cancelled' | 'unknown'
  tickets_left: number
}

export interface SaleInfo {
  event_pubkey: string
  registration_start: number
  registration_end: number
  reveal_at: number
  claim_start: number
  round_duration_secs: number
  stake_lamports: number
  window_size: number
  total_entries: number
  settled: boolean
  claimed: number
  settled_count: number
  forfeited_count: number
}

export interface QueueEntryInfo {
  pubkey: string
  buyer: string
  position: number
  effective_position: number
  round: number
  round_starts_at: number
  round_ends_at: number
  stake_lamports: number
  status: 'staked' | 'claimed' | 'settled' | 'forfeited' | 'unknown'
}

export interface QueueState extends SaleInfo {
  phase: 'announced' | 'registration' | 'reveal' | 'draw' | 'claim'
  pending: number
  current_round: number
  round_serving_from: number
  round_serving_to: number
  my_entry?: QueueEntryInfo
}

export interface WalletTicket {
  pubkey: string
  event_pubkey: string
  buyer: string
  mint: string
  status: 'valid' | 'used' | 'unknown'
  checked_in_at: number
  event_title: string
  event_starts_at: number
  event_ends_at: number
  event_city: string
  event_status: string
}

export interface EventDetails extends Omit<EventSummary, 'tickets_left' | 'status'> {
  status: 'active' | 'cancelled' | 'unknown'
  tickets_left: number
  sale?: SaleInfo
}

interface EventListQuery {
  city?: string
  q?: string
  status?: string
  organizer?: string
  upcoming?: boolean
  limit?: number
  offset?: number
}

async function get<T>(path: string): Promise<T> {
  const res = await fetch(path)
  if (!res.ok) {
    throw new Error(`api ${res.status}`)
  }
  return res.json() as Promise<T>
}

export const api = {
  events(params: EventListQuery = {}): Promise<{ events: EventSummary[]; total: number }> {
    const q = new URLSearchParams()
    for (const [k, v] of Object.entries(params)) {
      if (v !== '' && v !== false && v != null) q.set(k, String(v))
    }
    const qs = q.toString()
    return get(`/api/v1/events${qs ? `?${qs}` : ''}`)
  },
  event(pubkey: string): Promise<EventDetails> {
    return get(`/api/v1/events/${pubkey}`)
  },
  queue(pubkey: string, buyer?: string): Promise<QueueState> {
    const qs = buyer ? `?buyer=${buyer}` : ''
    return get(`/api/v1/events/${pubkey}/queue${qs}`)
  },
  walletTickets(pubkey: string): Promise<{ tickets: WalletTicket[] }> {
    return get(`/api/v1/wallets/${pubkey}/tickets`)
  },
  ticketByMint(mint: string): Promise<WalletTicket> {
    return get(`/api/v1/tickets/mint/${mint}`)
  },
}

export function lamportsToSol(lamports: number | bigint): string {
  return (Number(lamports) / 1e9).toFixed(2)
}

export function fmtDate(unixSec: number): string {
  return new Date(Number(unixSec) * 1000).toLocaleString('ru-RU', {
    day: '2-digit',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
  })
}

export function fmtCountdown(unixSec: number, nowSec: number): string {
  let left = Math.max(0, Number(unixSec) - nowSec)
  const d = Math.floor(left / 86400)
  left %= 86400
  const h = Math.floor(left / 3600)
  const m = Math.floor((left % 3600) / 60)
  const s = left % 60
  if (d > 0) return `${d}д ${h}ч`
  if (h > 0) return `${h}ч ${m}м`
  return `${m}м ${String(s).padStart(2, '0')}с`
}
