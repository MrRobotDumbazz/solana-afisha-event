import { Connection, clusterApiUrl } from '@solana/web3.js'

export const connection = new Connection(
  import.meta.env.VITE_RPC_URL || clusterApiUrl('devnet'),
  'confirmed',
)

async function get(path) {
  const res = await fetch(path)
  if (!res.ok) {
    throw new Error(`api ${res.status}`)
  }
  return res.json()
}

export const api = {
  events(params = {}) {
    const q = new URLSearchParams()
    for (const [k, v] of Object.entries(params)) {
      if (v !== '' && v !== false && v != null) q.set(k, v)
    }
    const qs = q.toString()
    return get(`/api/v1/events${qs ? `?${qs}` : ''}`)
  },
  event(pubkey) {
    return get(`/api/v1/events/${pubkey}`)
  },
  queue(pubkey, buyer) {
    const qs = buyer ? `?buyer=${buyer}` : ''
    return get(`/api/v1/events/${pubkey}/queue${qs}`)
  },
  walletTickets(pubkey) {
    return get(`/api/v1/wallets/${pubkey}/tickets`)
  },
  ticketByMint(mint) {
    return get(`/api/v1/tickets/mint/${mint}`)
  },
}

export function lamportsToSol(lamports) {
  return (Number(lamports) / 1e9).toFixed(2)
}

export function fmtDate(unixSec) {
  return new Date(Number(unixSec) * 1000).toLocaleString('ru-RU', {
    day: '2-digit',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
  })
}

export function fmtCountdown(unixSec, nowSec) {
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
