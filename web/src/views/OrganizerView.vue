<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { useWallet } from '@solana/wallet-adapter-vue'
import { PublicKey } from '@solana/web3.js'
import { api, fmtDate, lamportsToSol } from '../api'
import { ixInitEvent, ixConfigureSale } from '../solana/program'
import { useTransactions } from '../composables/transaction'

const { publicKey } = useWallet()
const { pending, error, send } = useTransactions()

const myEvents = ref([])
const created = ref(false)

const form = ref({
  slug: '',
  title: '',
  description: '',
  venue: '',
  city: '',
  image_uri: '',
  date: '',
  time: '',
  durationHours: 2,
  priceSol: 0.05,
  capacity: 100,
  hot_sale: false,
})

const saleForm = ref(null)
const saleEvent = ref(null)

async function loadMine() {
  if (!publicKey.value) {
    myEvents.value = []
    return
  }
  try {
    const data = await api.events({ organizer: publicKey.value.toBase58() })
    myEvents.value = data.events || []
  } catch {
    myEvents.value = []
  }
}

onMounted(loadMine)
watch(publicKey, loadMine)

const startsAt = computed(() => {
  const f = form.value
  if (!f.date || !f.time) return null
  return Math.floor(new Date(`${f.date}T${f.time}`).getTime() / 1000)
})

const canCreate = computed(
  () =>
    publicKey.value &&
    form.value.slug &&
    form.value.title &&
    form.value.venue &&
    form.value.city &&
    startsAt.value &&
    !pending.value,
)

async function createEvent() {
  created.value = false
  const f = form.value
  const params = {
    title: f.title,
    description: f.description || '',
    venue: f.venue,
    city: f.city,
    image_uri: f.image_uri || '',
    starts_at: BigInt(startsAt.value),
    ends_at: BigInt(startsAt.value + f.durationHours * 3600),
    ticket_price_lamports: BigInt(Math.round(f.priceSol * 1e9)),
    capacity: Number(f.capacity),
    hot_sale: f.hot_sale,
  }
  try {
    await send(publicKey, [ixInitEvent(publicKey.value, slugify(f.slug), params)])
    created.value = true
    form.value.slug = ''
    form.value.title = ''
    await loadMine()
  } catch {
  }
}

function slugify(s) {
  return s
    .toLowerCase()
    .replace(/[^a-z0-9-]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 32)
}

function openSaleConfig(ev) {
  saleEvent.value = ev
  const now = Math.floor(Date.now() / 1000)
  saleForm.value = {
    registration_start: now + 3600,
    registration_end: now + 86400,
    reveal_at: now + 90000,
    claim_start: now + 93600,
    round_duration_secs: 300,
    stake_sol: 0.05,
    window_size: 25,
  }
}

const canConfigure = computed(() => saleEvent.value && !pending.value)

async function configureSale() {
  const f = saleForm.value
  const ev = saleEvent.value
  try {
    await send(
      publicKey,
      [
        ixConfigureSale(publicKey.value, new PublicKey(ev.pubkey), ev.slug, {
          registration_start: BigInt(f.registration_start),
          registration_end: BigInt(f.registration_end),
          reveal_at: BigInt(f.reveal_at),
          claim_start: BigInt(f.claim_start),
          round_duration_secs: BigInt(f.round_duration_secs),
          stake_lamports: BigInt(Math.round(f.stake_sol * 1e9)),
          window_size: Number(f.window_size),
        }),
      ],
      { computeUnits: 200000 },
    )
    saleEvent.value = null
    await loadMine()
  } catch {
  }
}
</script>

<template>
  <section>
    <h2>Кабинет организатора</h2>
    <p v-if="!publicKey" class="muted">Подключите кошелёк.</p>

    <template v-else>
      <div class="card">
        <h3>Новое событие</h3>
        <div class="form">
          <label>Slug (латиница, для адреса PDA)<input v-model="form.slug" placeholder="my-show-2026" /></label>
          <label>Название<input v-model="form.title" /></label>
          <label>Описание<input v-model="form.description" /></label>
          <div class="row3">
            <label>Город<input v-model="form.city" /></label>
            <label>Место<input v-model="form.venue" /></label>
            <label>Картинка (URL)<input v-model="form.image_uri" /></label>
          </div>
          <div class="row4">
            <label>Дата<input type="date" v-model="form.date" /></label>
            <label>Время<input type="time" v-model="form.time" /></label>
            <label>Длит. (ч)<input type="number" min="1" v-model.number="form.durationHours" /></label>
            <label>Вместимость<input type="number" min="1" v-model.number="form.capacity" /></label>
          </div>
          <div class="row3">
            <label>Цена, SOL<input type="number" step="0.001" min="0" v-model.number="form.priceSol" /></label>
            <label class="check">
              <input type="checkbox" v-model="form.hot_sale" />
              Честная очередь (анти-боты)
            </label>
          </div>
          <button class="primary" :disabled="!canCreate" @click="createEvent">
            <template v-if="pending">Отправка…</template>
            <template v-else>Создать событие (on-chain)</template>
          </button>
          <p v-if="created" class="ok">Событие создано ончейн!</p>
          <p v-if="error" class="warn">{{ error }}</p>
        </div>
      </div>

      <div v-if="saleEvent" class="card">
        <h3>Настройка очереди: {{ saleEvent.title }}</h3>
        <div class="form">
          <div class="row4">
            <label>Начало регистрации<input type="number" v-model.number="saleForm.registration_start" /></label>
            <label>Конец регистрации<input type="number" v-model.number="saleForm.registration_end" /></label>
            <label>Reveal<input type="number" v-model.number="saleForm.reveal_at" /></label>
            <label>Старт продаж<input type="number" v-model.number="saleForm.claim_start" /></label>
          </div>
          <div class="row3">
            <label>Раунд, сек<input type="number" min="60" v-model.number="saleForm.round_duration_secs" /></label>
            <label>Окно, позиций<input type="number" min="1" v-model.number="saleForm.window_size" /></label>
            <label>Стейк, SOL<input type="number" step="0.001" min="0" v-model.number="saleForm.stake_sol" /></label>
          </div>
          <button class="primary" :disabled="!canConfigure" @click="configureSale">
            <template v-if="pending">Отправка…</template>
            <template v-else>Настроить очередь</template>
          </button>
          <button class="ghost" @click="saleEvent = null">Отмена</button>
        </div>
      </div>

      <h3>Мои события</h3>
      <p v-if="myEvents.length === 0" class="muted">Пока нет событий.</p>
      <div v-else class="list">
        <div v-for="e in myEvents" :key="e.pubkey" class="card row">
          <div>
            <strong>{{ e.title }}</strong>
            <p class="muted">{{ fmtDate(e.starts_at) }} · {{ e.city }} · {{ e.tickets_sold }}/{{ e.capacity }}</p>
          </div>
          <div class="actions">
            <RouterLink :to="`/event/${e.pubkey}`" class="nav-link">Открыть</RouterLink>
            <button
              v-if="e.hot_sale && !e.sale_configured"
              @click="openSaleConfig(e)"
            >
              Очередь…
            </button>
          </div>
        </div>
      </div>
    </template>
  </section>
</template>

<style scoped>
.form {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-width: 640px;
}
label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
  color: var(--muted);
}
input {
  background: var(--bg);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 8px;
  padding: 8px 10px;
  font: inherit;
}
.check {
  flex-direction: row;
  align-items: center;
  gap: 8px;
}
.row3 {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}
.row4 {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
}
.row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}
.actions {
  display: flex;
  gap: 8px;
  align-items: center;
}
.list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-width: 720px;
}
.nav-link {
  color: var(--accent2);
  text-decoration: none;
}
.ok {
  color: var(--accent);
}
.warn {
  color: #ff6b6b;
  word-break: break-all;
}
</style>
