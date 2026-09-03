<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useWallet } from '@solana/wallet-adapter-vue'
import { api, type WalletTicket } from '../api'
import TicketCard from '../components/TicketCard.vue'

const { publicKey } = useWallet()
const tickets = ref<WalletTicket[] | null>(null)
const error = ref<string | null>(null)

async function load() {
  if (!publicKey.value) {
    tickets.value = null
    return
  }
  error.value = null
  try {
    const data = await api.walletTickets(publicKey.value.toBase58())
    tickets.value = data.tickets || []
  } catch {
    error.value = 'Не удалось загрузить билеты'
  }
}

onMounted(load)
watch(publicKey, load)
</script>

<template>
  <section>
    <h2>Мои билеты</h2>
    <p v-if="!publicKey" class="muted">Подключите кошелёк, чтобы увидеть билеты.</p>
    <p v-else-if="error" class="muted">{{ error }}</p>
    <p v-else-if="tickets === null" class="muted">Загрузка…</p>
    <p v-else-if="tickets.length === 0" class="muted">
      Билетов нет — NFT-билеты появятся здесь после покупки.
    </p>
    <div v-else class="list">
      <TicketCard v-for="t in tickets" :key="t.pubkey" :ticket="t" />
    </div>
  </section>
</template>

<style scoped>
h2 {
  margin-top: 0;
}
.list {
  display: flex;
  flex-direction: column;
  gap: 14px;
  max-width: 640px;
}
</style>
