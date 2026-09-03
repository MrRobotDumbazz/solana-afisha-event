declare module '@solana/wallet-adapter-vue' {
  import type { Ref } from 'vue'
  import type {
    Adapter,
    MessageSignerWalletAdapter,
    SendTransactionOptions,
    SignerWalletAdapter,
    Wallet,
    WalletError,
    WalletName,
  } from '@solana/wallet-adapter-base'
  import type {
    Connection,
    PublicKey,
    Transaction,
    TransactionSignature,
  } from '@solana/web3.js'

  export interface WalletStore {
    wallets: Wallet[]
    autoConnect: boolean
    wallet: Ref<Wallet | null>
    adapter: Ref<Adapter | null>
    publicKey: Ref<PublicKey | null>
    ready: Ref<boolean>
    connected: Ref<boolean>
    connecting: Ref<boolean>
    disconnecting: Ref<boolean>
    select(walletName: WalletName): void
    connect(): Promise<void>
    disconnect(): Promise<void>
    sendTransaction(
      transaction: Transaction,
      connection: Connection,
      options?: SendTransactionOptions,
    ): Promise<TransactionSignature>
    signTransaction: Ref<SignerWalletAdapter['signTransaction'] | undefined>
    signAllTransactions: Ref<SignerWalletAdapter['signAllTransactions'] | undefined>
    signMessage: Ref<MessageSignerWalletAdapter['signMessage'] | undefined>
  }

  export interface WalletStoreProps {
    wallets: Adapter[]
    autoConnect?: boolean
    onError?: (error: WalletError) => void
    localStorageKey?: string
  }

  export function useWallet(): WalletStore
  export function initWallet(walletStoreProps: WalletStoreProps): void
  export function provideWallet(walletStoreProps: WalletStoreProps): void
}
