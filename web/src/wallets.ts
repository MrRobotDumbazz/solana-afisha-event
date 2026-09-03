import {
  PhantomWalletAdapter,
} from '@solana/wallet-adapter-phantom'
import { SolflareWalletAdapter } from '@solana/wallet-adapter-solflare'
import { TrustWalletAdapter } from '@solana/wallet-adapter-trust'
import { CoinbaseWalletAdapter } from '@solana/wallet-adapter-coinbase'
import { SkyWalletAdapter } from '@solana/wallet-adapter-sky'
import { SafePalWalletAdapter } from '@solana/wallet-adapter-safepal'
import { NufiWalletAdapter } from '@solana/wallet-adapter-nufi'
import type { Adapter } from '@solana/wallet-adapter-base'

export function buildWalletAdapters(): Adapter[] {
  const adapters: Adapter[] = []
  const candidates: (() => Adapter)[] = [
    () => new PhantomWalletAdapter(),
    () => new SolflareWalletAdapter(),
    () => new TrustWalletAdapter(),
    () => new CoinbaseWalletAdapter(),
    () => new SkyWalletAdapter(),
    () => new SafePalWalletAdapter(),
    () => new NufiWalletAdapter(),
  ]
  for (const make of candidates) {
    try {
      adapters.push(make())
    } catch {
      // адаптер требует окружение, которого нет (например, расширение) — пропускаем
    }
  }
  return adapters
}
