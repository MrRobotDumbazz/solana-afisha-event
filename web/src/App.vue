<script setup>
import { useWallet } from '@solana/wallet-adapter-vue'
import ConnectWallet from './components/ConnectWallet.vue'
import EventList from './components/EventList.vue'

const { publicKey } = useWallet()

function shortAddress(address) {
  return `${address.slice(0, 4)}…${address.slice(-4)}`
}
</script>

<template>
  <header class="header">
    <h1>Afisha<span class="accent">.sol</span></h1>
    <div>
      <span v-if="publicKey" class="muted">{{ shortAddress(publicKey.toBase58()) }}</span>
      <ConnectWallet />
    </div>
  </header>
  <main>
    <EventList />
  </main>
</template>
