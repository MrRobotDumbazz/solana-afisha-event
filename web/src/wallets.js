import { PhantomWalletAdapter } from '@solana/wallet-adapter-phantom'
import { SolflareWalletAdapter } from '@solana/wallet-adapter-solflare'
import { TrustWalletAdapter } from '@solana/wallet-adapter-trust'
import { CoinbaseWalletAdapter } from '@solana/wallet-adapter-coinbase'
import { SkyWalletAdapter } from '@solana/wallet-adapter-sky'
import { SafePalWalletAdapter } from '@solana/wallet-adapter-safepal'
import { NufiWalletAdapter } from '@solana/wallet-adapter-nufi'

export function buildWalletAdapters() {
  const adapters = []
  try {
    adapters.push(new PhantomWalletAdapter())
  } catch {}
  try {
    adapters.push(new SolflareWalletAdapter())
  } catch {}
  try {
    adapters.push(new TrustWalletAdapter())
  } catch {}
  try {
    adapters.push(new CoinbaseWalletAdapter())
  } catch {}
  try {
    adapters.push(new SkyWalletAdapter())
  } catch {}
  try {
    adapters.push(new SafePalWalletAdapter())
  } catch {}
  try {
    adapters.push(new NufiWalletAdapter())
  } catch {}
  return adapters
}
