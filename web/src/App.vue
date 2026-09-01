<script setup>
import { useWallet } from '@solana/wallet-adapter-vue'
import { RouterLink } from 'vue-router'
import ConnectWallet from './components/ConnectWallet.vue'

const { publicKey } = useWallet()

function short(address) {
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
      <span v-if="publicKey" class="muted">{{ short(publicKey.toBase58()) }}</span>
      <ConnectWallet />
    </div>
  </header>
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
</style>
