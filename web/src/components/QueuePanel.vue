<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { PublicKey } from '@solana/web3.js'
import { useWallet } from '@solana/wallet-adapter-vue'
import { api, fmtCountdown, fmtDate, lamportsToSol } from '../api'
import { ixJoinQueue } from '../solana/program'
import { useTransactions } from '../composables/transaction'

const props = defineProps({
  eventPubkey: { type: String, required: true },
  buyer: { type: String, default: null },
  now: { type: Number, required: true },
  nonce: { type: Number, default: 0 },
})

const emit = defineEmits(['state'])

const { publicKey } = useWallet()
const { pending, error, send } = useTransactions()

const queue = ref(null)
const joined = ref(false)

let timer

async function load() {
  try {
    queue.value = await api.queue(props.eventPubkey, props.buyer || undefined)
  } catch {
    queue.value = null
  }
}

watch(
  () => props.nonce,
  () => load(),
)
watch(
  () => props.buyer,
  () => load(),
)

onMounted(() => {
  load()
  timer = setInterval(load, 7000)
})
onUnmounted(() => clearInterval(timer))

const q = computed(() => queue.value)
const myEntry = computed(() => q.value?.my_entry || null)

const phaseLabel = computed(() => {
  switch (q.value?.phase) {
    case 'announced':
      return 'Регистрация в очереди откроется'
    case 'registration':
      return 'Регистрация в очереди идёт, до конца'
    case 'reveal':
      return 'Жеребьёвка, результаты через'
    case 'draw':
      return 'Продажи по очереди начнутся через'
    case 'claim':
      return 'Идут продажи по очереди'
    default:
      return ''
  }
})

const phaseTarget = computed(() => {
  if (!q.value) return null
  switch (q.value.phase) {
    case 'announced':
      return q.value.registration_start
    case 'registration':
      return q.value.registration_end
    case 'reveal':
      return q.value.reveal_at
    case 'draw':
      return q.value.claim_start
    default:
      return null
  }
})

const myRoundActive = computed(() => {
  const e = myEntry.value
  if (!e || e.status !== 'staked') return false
  return props.now >= e.round_starts_at && props.now < e.round_ends_at
})

watch(myRoundActive, (v) => emitState(), { immediate: true })
watch(
  () => myEntry.value?.status,
  () => emitState(),
  { immediate: true },
)

function emitState() {
  emit('state', {
    myRoundActive: myRoundActive.value,
    entry: myEntry.value,
  })
}

const canJoin = computed(
  () =>
    q.value?.phase === 'registration' &&
    props.buyer &&
    !myEntry.value &&
    !pending.value,
)

async function join() {
  try {
    await send(publicKey, [ixJoinQueue(publicKey.value, new PublicKey(props.eventPubkey))])
    joined.value = true
    await load()
  } catch {
  }
}
</script>

<template>
  <div class="card queue">
    <h4>Честная очередь</h4>

    <template v-if="q">
      <p class="phase">
        {{ phaseLabel }}
        <strong v-if="phaseTarget">{{ fmtCountdown(phaseTarget, now) }}</strong>
      </p>

      <p class="muted">
        участников: {{ q.total_entries }} · выкупили: {{ q.claimed }} · ждут: {{ q.pending }}
      </p>

      <p v-if="q.phase === 'claim' && q.round_serving_to > q.round_serving_from" class="serving">
        сейчас обслуживают позиции №{{ q.round_serving_from }}–№{{ q.round_serving_to - 1 }}
      </p>

      <template v-if="myEntry">
        <div class="my card-inner">
          <p>
            Ваша позиция: <strong>№{{ myEntry.effective_position + 1 }}</strong> из
            {{ q.total_entries }}
            <span class="muted">(тикет #{{ myEntry.position + 1 }} в регистрации)</span>
          </p>
          <p>
            Ваше окно:
            <strong>{{ fmtDate(myEntry.round_starts_at) }} – {{ fmtDate(myEntry.round_ends_at) }}</strong>
          </p>
          <p>
            Статус:
            <strong :class="myEntry.status">{{ myEntry.status }}</strong>
            · стейк {{ lamportsToSol(myEntry.stake_lamports) }} SOL
          </p>
          <p v-if="myRoundActive" class="ok">Ваше окно активно — можно покупать!</p>
          <p v-else-if="myEntry.status === 'staked'" class="muted">
            до вашего окна {{ fmtCountdown(myEntry.round_starts_at, now) }}
          </p>
          <p v-if="myEntry.status === 'forfeited'" class="warn">
            Окно пропущено — стейк сгорел
          </p>
        </div>
      </template>

      <button v-if="canJoin" class="primary" :disabled="pending" @click="join">
        <template v-if="pending">Отправка…</template>
        <template v-else>
          Встать в очередь (стейк {{ lamportsToSol(q.stake_lamports) }} SOL)
        </template>
      </button>
      <p v-if="joined" class="ok">Вы в очереди!</p>
      <p v-if="!buyer" class="muted">Подключите кошелёк, чтобы участвовать</p>
      <p v-if="error" class="warn">{{ error }}</p>
    </template>
    <p v-else class="muted">Очередь ещё не настроена организатором</p>
  </div>
</template>

<style scoped>
.queue {
  margin: 16px 0;
}
h4 {
  margin: 0 0 10px;
}
.phase {
  margin: 0 0 6px;
}
.card-inner {
  margin-top: 12px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.card-inner p {
  margin: 0;
}
.serving {
  color: var(--accent2);
}
.ok {
  color: var(--accent);
}
.warn {
  color: #ff6b6b;
}
.claimed {
  color: var(--accent);
}
.forfeited {
  color: #ff6b6b;
}
.settled {
  color: var(--muted);
}
</style>
