<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { PublicKey } from '@solana/web3.js'
import { useWalletStore } from '../wallet/store'
import { api, fmtDate, lamportsToSol, type EventDetails } from '../api'
import { ixBuyTicket } from '../solana/program'
import { useTransactions } from '../composables/transaction'
import QueuePanel from '../components/QueuePanel.vue'

const props = defineProps({
  pubkey: { type: String, required: true },
})

const wallet = useWalletStore()
const { publicKey } = wallet
const { pending, error, send } = useTransactions()

const event = ref<EventDetails | null>(null)
const error404 = ref(false)
const justBought = ref(false)
const now = ref(Math.floor(Date.now() / 1000))
const myRoundActive = ref(false)
const queueNonce = ref(0)

let timer: ReturnType<typeof setInterval> | undefined

async function load() {
  try {
    event.value = await api.event(props.pubkey)
  } catch {
    error404.value = true
  }
}

function onQueueState(state: { myRoundActive: boolean }) {
  myRoundActive.value = state?.myRoundActive || false
}

const buyerKey = computed(() => publicKey.value?.toBase58() || null)

const canBuy = computed(() => {
  if (!event.value || !buyerKey.value || pending.value) return false
  const e = event.value
  if (e.status !== 'active') return false
  if (e.tickets_sold >= e.capacity) return false
  if (now.value >= e.starts_at) return false
  if (e.hot_sale) return myRoundActive.value
  return true
})

async function buy() {
  justBought.value = false
  try {
    await send(wallet, [
      ixBuyTicket(publicKey.value!, new PublicKey(props.pubkey), {
        hot: event.value!.hot_sale,
      }),
    ])
    justBought.value = true
    await load()
    queueNonce.value++
  } catch {
  }
}

onMounted(() => {
  load()
  timer = setInterval(() => (now.value = Math.floor(Date.now() / 1000)), 1000)
})
onUnmounted(() => clearInterval(timer))
</script>

<template>
  <section v-if="error404">
    <p class="muted">Событие не найдено</p>
  </section>
  <section v-else-if="!event">
    <p class="muted">Загрузка…</p>
  </section>
  <article v-else>
    <h2>{{ event.title }}</h2>
    <p class="muted">
      {{ fmtDate(event.starts_at) }} – {{ fmtDate(event.ends_at) }}<br />
      {{ event.city }}, {{ event.venue }}
    </p>
    <p v-if="event.description">{{ event.description }}</p>

    <div class="card meta">
      <div class="meta-row">
        <span>Цена</span>
        <strong>{{ lamportsToSol(event.ticket_price_lamports) }} SOL</strong>
      </div>
      <div class="meta-row">
        <span>Билетов</span>
        <strong>{{ event.tickets_sold }} / {{ event.capacity }}</strong>
      </div>
      <div class="meta-row">
        <span>Статус</span>
        <strong :class="event.status">{{ event.status }}</strong>
      </div>
      <div class="meta-row">
        <span>Организатор</span>
        <span class="mono">{{ event.organizer.slice(0, 10) }}…</span>
      </div>
    </div>

    <QueuePanel
      v-if="event.hot_sale"
      :event-pubkey="event.pubkey"
      :buyer="buyerKey ?? undefined"
      :now="now"
      :nonce="queueNonce"
      @state="onQueueState"
    />

    <div class="buy-row">
      <button class="primary big" :disabled="!canBuy" @click="buy">
        <template v-if="pending">Отправка транзакции…</template>
        <template v-else-if="event.tickets_sold >= event.capacity">Sold out</template>
        <template v-else-if="event.hot_sale && !myRoundActive && buyerKey">
          Ожидайте своего окна очереди
        </template>
        <template v-else-if="!buyerKey">Подключите кошелёк</template>
        <template v-else>Купить билет NFT</template>
      </button>
      <p v-if="error" class="error">{{ error }}</p>
      <p v-if="justBought" class="ok">Билет куплен! Смотрите «Мои билеты».</p>
    </div>
  </article>
</template>

<style scoped>
h2 {
  margin: 0 0 4px;
}
.meta {
  margin: 16px 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.meta-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}
.meta-row span:first-child {
  color: var(--muted);
}
.mono {
  font-family: monospace;
}
.buy-row {
  margin-top: 20px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.big {
  font-size: 16px;
  padding: 14px 20px;
}
.error {
  color: #ff6b6b;
  margin: 0;
  word-break: break-all;
}
.ok {
  color: var(--accent);
  margin: 0;
}
.active {
  color: var(--accent);
}
.cancelled {
  color: #ff6b6b;
}
</style>
