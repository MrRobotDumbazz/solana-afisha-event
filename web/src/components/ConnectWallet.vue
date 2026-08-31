<script setup>
import { useWallet } from '@solana/wallet-adapter-vue'

const { wallets, select, disconnect, publicKey, connecting } = useWallet()
</script>

<template>
  <div class="connect">
    <template v-if="!publicKey">
      <button
        v-for="w in wallets"
        :key="w.adapter.name"
        :disabled="connecting"
        @click="select(w.adapter.name)"
      >
        {{ w.adapter.name }}
      </button>
      <span v-if="wallets.length === 0" class="muted">Кошелёк не найден</span>
    </template>
    <button v-else @click="disconnect()">Выйти</button>
  </div>
</template>

<style scoped>
.connect {
  display: inline-flex;
  gap: 8px;
  margin-left: 12px;
}
</style>
