import { ref, watch, type Ref } from 'vue'
import { useWallet } from '@solana/wallet-adapter-vue'

const token = ref<string | null>(null)
const authWallet = ref<string | null>(null)

const TOKEN_KEY = 'afisha_auth_token'
const WALLET_KEY = 'afisha_auth_wallet'

function loadStored() {
  try {
    token.value = localStorage.getItem(TOKEN_KEY)
    authWallet.value = localStorage.getItem(WALLET_KEY)
  } catch {
    /* localStorage недоступен */
  }
}
loadStored()

export function useAuth() {
  const { publicKey, signMessage } = useWallet()

  const isAuthed: Ref<boolean> = ref(
    Boolean(token.value && authWallet.value && publicKey.value?.toBase58() === authWallet.value),
  )

  watch(publicKey, (pk) => {
    isAuthed.value = Boolean(pk && token.value && authWallet.value === pk.toBase58())
  })

  const authError = ref<string | null>(null)
  const signingIn = ref(false)

  async function signIn(): Promise<void> {
    if (!publicKey.value) {
      authError.value = 'Сначала подключите кошелёк'
      throw new Error(authError.value)
    }
    authError.value = null
    signingIn.value = true
    try {
      const nonceRes = await fetch('/api/v1/auth/nonce', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet: publicKey.value.toBase58() }),
      })
      if (!nonceRes.ok) throw new Error('не удалось получить nonce')
      const { nonce, message } = (await nonceRes.json()) as { nonce: string; message: string }

      const signFn = signMessage.value
      if (!signFn) {
        throw new Error('Кошелёк не поддерживает подпись сообщений')
      }
      const encoded = new TextEncoder().encode(message)
      const signature: Uint8Array = await signFn(encoded)
      const sigB64 = btoa(String.fromCharCode(...signature))

      const verifyRes = await fetch('/api/v1/auth/verify', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          wallet: publicKey.value.toBase58(),
          nonce,
          signature: sigB64,
        }),
      })
      if (!verifyRes.ok) {
        const body = (await verifyRes.json().catch(() => ({}))) as { error?: string }
        throw new Error(body.error || 'подпись отклонена')
      }
      const data = (await verifyRes.json()) as { token: string; wallet: string }
      token.value = data.token
      authWallet.value = data.wallet
      localStorage.setItem(TOKEN_KEY, data.token)
      localStorage.setItem(WALLET_KEY, data.wallet)
      isAuthed.value = true
    } catch (e) {
      authError.value = e instanceof Error ? e.message : 'ошибка входа'
      throw e
    } finally {
      signingIn.value = false
    }
  }

  function signOut(): void {
    token.value = null
    authWallet.value = null
    isAuthed.value = false
    localStorage.removeItem(TOKEN_KEY)
    localStorage.removeItem(WALLET_KEY)
  }

  async function authFetch(path: string, options: RequestInit = {}): Promise<Response> {
    const headers = new Headers(options.headers || {})
    if (token.value) headers.set('Authorization', `Bearer ${token.value}`)
    return fetch(path, { ...options, headers })
  }

  return { isAuthed, authWallet, authError, signingIn, signIn, signOut, authFetch }
}
