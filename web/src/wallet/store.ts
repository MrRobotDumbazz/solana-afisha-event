import { reactive, ref, type Ref } from 'vue'
import type {
  Adapter,
  WalletError,
  WalletReadyState,
} from '@solana/wallet-adapter-base'
import { WalletReadyState as ReadyState } from '@solana/wallet-adapter-base'
import type { Connection, PublicKey, Transaction, TransactionSignature } from '@solana/web3.js'

export interface WalletEntry {
  adapter: Adapter
  readyState: WalletReadyState
}

const entries = reactive<WalletEntry[]>([])
const selectedName = ref<string | null>(null)
const publicKey = ref<PublicKey | null>(null)
const connected = ref(false)
const connecting = ref(false)
let currentAdapter: Adapter | null = null
let boundAdapter: Adapter | null = null
let autoConnectAttempted = false

function persistName(name: string | null) {
  try {
    if (name) localStorage.setItem('afisha_wallet_name', name)
    else localStorage.removeItem('afisha_wallet_name')
  } catch {
    /* ignore */
  }
}

function storedName(): string | null {
  try {
    return localStorage.getItem('afisha_wallet_name')
  } catch {
    return null
  }
}

function syncState() {
  publicKey.value = currentAdapter?.publicKey ?? null
  connected.value = currentAdapter?.connected ?? false
}

function bindEvents(adapter: Adapter) {
  unbindEvents()
  boundAdapter = adapter
  adapter.on('connect', syncState)
  adapter.on('disconnect', onAdapterDisconnect)
}

function unbindEvents() {
  if (!boundAdapter) return
  boundAdapter.off('connect', syncState)
  boundAdapter.off('disconnect', onAdapterDisconnect)
  boundAdapter = null
}

async function onAdapterDisconnect() {
  publicKey.value = null
  connected.value = false
}

export function initWalletStore(adapters: Adapter[], onError?: (e: WalletError) => void) {
  const handle = (e: WalletError) => onError?.(e)
  for (const adapter of adapters) {
    const entry = reactive({ adapter, readyState: adapter.readyState }) as WalletEntry
    adapter.on('readyStateChange', (state: WalletReadyState) => {
      entry.readyState = state
    })
    adapter.on('error', handle)
    entries.push(entry)
  }
  void tryAutoConnect()
}

async function tryAutoConnect() {
  if (autoConnectAttempted) return
  autoConnectAttempted = true
  const name = storedName()
  if (!name) return
  const entry = entries.find((e) => e.adapter.name === name)
  if (!entry || entry.readyState !== ReadyState.Installed) return
  try {
    await select(name)
  } catch {
    persistName(null)
  }
}

export async function select(name: string): Promise<void> {
  const entry = entries.find((e) => e.adapter.name === name)
  if (!entry) {
    throw new Error(`Кошелёк ${name} не найден`)
  }
  await disconnect()
  currentAdapter = entry.adapter
  selectedName.value = name
  persistName(name)
  bindEvents(currentAdapter)
  syncState()

  if (entry.readyState !== ReadyState.Installed) {
    window.open(entry.adapter.url, '_blank')
    return
  }
  connecting.value = true
  try {
    await currentAdapter.connect()
  } catch (e) {
    await disconnect()
    throw e
  } finally {
    connecting.value = false
  }
  syncState()
}

export async function disconnect(): Promise<void> {
  unbindEvents()
  const adapter = currentAdapter
  currentAdapter = null
  selectedName.value = null
  persistName(null)
  publicKey.value = null
  connected.value = false
  if (adapter?.connected) {
    try {
      await adapter.disconnect()
    } catch {
      /* ignore */
    }
  }
}

export async function sendTransaction(
  transaction: Transaction,
  connection: Connection,
): Promise<TransactionSignature> {
  if (!currentAdapter) throw new Error('Кошелёк не подключён')
  return currentAdapter.sendTransaction(transaction, connection)
}

export async function signMessage(message: Uint8Array): Promise<Uint8Array> {
  if (!currentAdapter) throw new Error('Кошелёк не подключён')
  const signer = currentAdapter as Adapter & {
    signMessage?: (message: Uint8Array) => Promise<Uint8Array>
  }
  if (!signer.signMessage) {
    throw new Error('Кошелёк не поддерживает подпись сообщений')
  }
  return signer.signMessage(message)
}

export interface WalletStoreApi {
  wallets: WalletEntry[]
  walletName: Ref<string | null>
  publicKey: Ref<PublicKey | null>
  connected: Ref<boolean>
  connecting: Ref<boolean>
  select: typeof select
  disconnect: typeof disconnect
  sendTransaction: typeof sendTransaction
  signMessage: typeof signMessage
}

export function useWalletStore(): WalletStoreApi {
  return {
    wallets: entries,
    walletName: selectedName,
    publicKey,
    connected,
    connecting,
    select,
    disconnect,
    sendTransaction,
    signMessage,
  }
}
