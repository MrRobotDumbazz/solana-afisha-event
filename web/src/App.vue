<script setup lang="ts">
import { useWallet } from '@solana/wallet-adapter-vue'
import { RouterLink } from 'vue-router'
import ConnectWallet from './components/ConnectWallet.vue'
import { useAuth } from './composables/auth'

const { publicKey } = useWallet()
const { isAuthed, signIn, signOut, signingIn, authError } = useAuth()

function short(address: string) {
  return `${address.slice(0, 4)}…${address.slice(-4)}`
}
</script>

<template>
  <header class="header">
    <nav class="nav">
      <RouterLink to="/" class="brand">Afisha<span class="accent">.sol</span></RouterLink>
      <RouterLink to="/tickets" class="nav-link">Мои билеты</RouterLink>
      <RouterLink to="/organizer" class="nav-link">Организатор</RouterLink>
      <RouterLink to="/scan" class="nav-link">Сканер</RouterLink>
    </nav>
    <div class="wallet-box">
      <span v-if="isAuthed" class="auth-badge" title="Вход подтверждён подписью">✓ вход</span>
      <span v-if="publicKey" class="muted">{{ short(publicKey.toBase58()) }}</span>

      <button
        v-if="publicKey && !isAuthed"
        class="signin"
        :disabled="signingIn"
        @click="signIn()"
      >
        <template v-if="signingIn">Подпишите…</template>
        <template v-else>Войти</template>
      </button>
      <button v-if="isAuthed" class="signin" @click="signOut()">Выйти</button>

      <ConnectWallet />
    </div>
  </header>
  <p v-if="authError" class="auth-error">{{ authError }}</p>
  <main>
    <RouterView />
  </main>
</template>

<style scoped>
.nav {
  display: flex;
  align-items: center;
  gap: 20px;
}
.brand {
  font-size: 20px;
  font-weight: 700;
  color: var(--text);
  text-decoration: none;
}
.nav-link {
  color: var(--muted);
  text-decoration: none;
}
.nav-link:hover {
  color: var(--accent);
}
.wallet-box {
  display: flex;
  align-items: center;
  gap: 10px;
}
.auth-badge {
  color: var(--accent);
  font-size: 13px;
  border: 1px solid var(--accent);
  border-radius: 999px;
  padding: 2px 10px;
}
.signin {
  padding: 6px 12px;
}
.auth-error {
  color: #ff6b6b;
  margin: 0;
  padding: 8px 24px;
  font-size: 13px;
}
</style>
