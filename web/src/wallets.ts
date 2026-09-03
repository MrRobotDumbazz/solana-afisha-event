import type { Adapter } from '@solana/wallet-adapter-base'
import { PhantomWalletAdapter } from '@solana/wallet-adapter-phantom'
import { SolflareWalletAdapter } from '@solana/wallet-adapter-solflare'
import { BackpackWalletAdapter } from '@solana/wallet-adapter-backpack'
import { TrustWalletAdapter } from '@solana/wallet-adapter-trust'
import { CoinbaseWalletAdapter } from '@solana/wallet-adapter-coinbase'
import { GlowWalletAdapter } from '@solana/wallet-adapter-glow'
import { ExodusWalletAdapter } from '@solana/wallet-adapter-exodus'
import { BraveWalletAdapter } from '@solana/wallet-adapter-brave'
import { SkyWalletAdapter } from '@solana/wallet-adapter-sky'
import { SafePalWalletAdapter } from '@solana/wallet-adapter-safepal'
import { NufiWalletAdapter } from '@solana/wallet-adapter-nufi'

export function buildWalletAdapters(): Adapter[] {
  const candidates: (() => Adapter)[] = [
    () => new PhantomWalletAdapter(),
    () => new SolflareWalletAdapter(),
    () => new BackpackWalletAdapter(),
    () => new TrustWalletAdapter(),
    () => new CoinbaseWalletAdapter(),
    () => new GlowWalletAdapter(),
    () => new ExodusWalletAdapter(),
    () => new BraveWalletAdapter(),
    () => new SkyWalletAdapter(),
    () => new SafePalWalletAdapter(),
    () => new NufiWalletAdapter(),
  ]
  const adapters: Adapter[] = []
  for (const make of candidates) {
    try {
      adapters.push(make())
    } catch {
      // адаптер требует окружение, которого нет — пропускаем
    }
  }
  return adapters
}
