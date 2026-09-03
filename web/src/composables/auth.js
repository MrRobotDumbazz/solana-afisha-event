import { ref, watch } from 'vue'
import { useWallet } from '@solana/wallet-adapter-vue'

const token = ref(null)
const authWallet = ref(null)
const authError = ref(null)
const signingIn = ref(false)

const TOKEN_KEY = 'afisha_auth_token'
const WALLET_KEY = 'afisha_auth_wallet'

function loadStored() {
  try {
    token.value = localStorage.getItem(TOKEN_KEY) || null
    authWallet.value = localStorage.getItem(WALLET_KEY) || null
  } catch {}
}
loadStored()

export function useAuth() {
  const { publicKey, signMessage } = useWallet()

  const isAuthed = ref(
    Boolean(token.value && authWallet.value && authWallet.value === publicKey.value?.toBase58()),
  )

  watch(publicKey, (pk) => {
    isAuthed.value = Boolean(pk && token.value && authWallet.value === pk.toBase58())
  })

  async function signIn() {
    if (!publicKey.value) {
      authError.value = 'Сначала подключите кошелёк'
      return
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
      const { nonce, message } = await nonceRes.json()

      const encoded = new TextEncoder().encode(message)
      const signature = await signMessage.value(encoded, 'utf8')

      const sigB64 = btoa(String.fromCharCode(...new Uint8Array(signature)))

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
        const body = await verifyRes.json().catch(() => ({}))
        throw new Error(body.error || 'подпись отклонена')
      }
      const data = await verifyRes.json()
      token.value = data.token
      authWallet.value = data.wallet
      localStorage.setItem(TOKEN_KEY, data.token)
      localStorage.setItem(WALLET_KEY, data.wallet)
      isAuthed.value = true
    } catch (e) {
      authError.value = e?.message || 'ошибка входа'
      throw e
    } finally {
      signingIn.value = false
    }
  }

  function signOut() {
    token.value = null
    authWallet.value = null
    isAuthed.value = false
    localStorage.removeItem(TOKEN_KEY)
    localStorage.removeItem(WALLET_KEY)
  }

  async function authFetch(path, options = {}) {
    const headers = new Headers(options.headers || {})
    if (token.value) headers.set('Authorization', `Bearer ${token.value}`)
    return fetch(path, { ...options, headers })
  }

  return { isAuthed, authWallet, authError, signingIn, signIn, signOut, authFetch }
}
