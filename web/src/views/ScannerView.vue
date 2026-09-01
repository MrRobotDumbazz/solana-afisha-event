<script setup>
import { onBeforeUnmount, onMounted, ref } from 'vue'
import jsQR from 'jsqr'
import { PublicKey } from '@solana/web3.js'
import { useWallet } from '@solana/wallet-adapter-vue'
import { api, fmtDate } from '../api'
import { ixCheckIn } from '../solana/program'
import { useTransactions } from '../composables/transaction'

const { publicKey } = useWallet()
const { pending, error, send } = useTransactions()

const video = ref(null)
const canvas = ref(null)
const scanning = ref(false)
const status = ref('Наведите камеру на QR-код билета')
const ticket = ref(null)
const ticketError = ref(null)
const checkedIn = ref(false)
const notOrganizer = ref(false)

let stream = null
let raf = 0

onMounted(start)
onBeforeUnmount(stop)

async function start() {
  try {
    stream = await navigator.mediaDevices.getUserMedia({
      video: { facingMode: 'environment' },
    })
    video.value.srcObject = stream
    await video.value.play()
    scanning.value = true
    tick()
  } catch {
    status.value = 'Нет доступа к камере — введите mint вручную'
  }
}

function stop() {
  cancelAnimationFrame(raf)
  scanning.value = false
  if (stream) stream.getTracks().forEach((t) => t.stop())
}

function tick() {
  if (!scanning.value) return
  const v = video.value
  const c = canvas.value
  if (v.readyState === v.HAVE_ENOUGH_DATA && c) {
    c.width = v.videoWidth
    c.height = v.videoHeight
    const ctx = c.getContext('2d', { willReadFrequently: true })
    ctx.drawImage(v, 0, 0, c.width, c.height)
    const image = ctx.getImageData(0, 0, c.width, c.height)
    const code = jsQR(image.data, image.width, image.height, { inversionAttempts: 'dontInvert' })
    if (code?.data && code.data.length >= 32 && code.data.length <= 44) {
      handleScan(code.data)
      return
    }
  }
  raf = requestAnimationFrame(tick)
}

async function handleScan(mint) {
  stop()
  status.value = `QR: ${mint}`
  ticketError.value = null
  ticket.value = null
  checkedIn.value = false
  notOrganizer.value = false
  try {
    const t = await api.ticketByMint(mint)
    ticket.value = t
    const ev = await api.event(t.event_pubkey)
    ticket.value._slug = ev.slug
    ticket.value._event = ev
    notOrganizer.value = ev.organizer !== publicKey.value?.toBase58()
  } catch {
    ticketError.value = 'Билет не найден в индексе'
  }
  setTimeout(() => {
    if (scanning.value === false && !ticket.value) start()
  }, 3000)
}

async function checkIn() {
  const t = ticket.value
  try {
    await send(
      publicKey,
      [
        ixCheckIn(
          publicKey.value,
          new PublicKey(t.event_pubkey),
          new PublicKey(t.pubkey),
          t._slug,
        ),
      ],
      { computeUnits: 200000 },
    )
    checkedIn.value = true
    const fresh = await api.ticketByMint(t.mint)
    fresh._slug = t._slug
    fresh._event = t._event
    ticket.value = fresh
  } catch {
  }
}

function reset() {
  ticket.value = null
  ticketError.value = null
  checkedIn.value = false
  status.value = 'Сканирование…'
  start()
}
</script>

<template>
  <section>
    <h2>Сканер билетов</h2>
    <p v-if="!publicKey" class="muted">Подключите кошелёк организатора.</p>

    <template v-else>
      <div class="scan-box card">
        <video ref="video" playsinline muted></video>
        <canvas ref="canvas" class="hidden"></canvas>
        <p class="muted">{{ status }}</p>
      </div>

      <div v-if="ticketError" class="card warn-card">
        <p class="warn">{{ ticketError }}</p>
        <button @click="reset">Сканировать снова</button>
      </div>

      <div v-if="ticket" class="card result">
        <h3>{{ ticket.event_title }}</h3>
        <p class="muted">{{ fmtDate(ticket.event_starts_at) }} · {{ ticket.event_city }}</p>
        <p>
          Покупатель:
          <span class="mono">{{ ticket.buyer }}</span>
        </p>
        <p>
          Статус:
          <strong :class="ticket.status">
            {{ ticket.status === 'used' ? 'уже использован' : 'действителен' }}
          </strong>
        </p>

        <p v-if="notOrganizer" class="warn">Вы не организатор этого события</p>

        <button
          v-else-if="ticket.status === 'valid' && !checkedIn"
          class="primary"
          :disabled="pending"
          @click="checkIn"
        >
          <template v-if="pending">Отправка…</template>
          <template v-else>Отметить вход (on-chain)</template>
        </button>
        <p v-if="checkedIn" class="ok">Вход отмечен ✓</p>
        <p v-if="error" class="warn">{{ error }}</p>
        <button class="ghost" @click="reset">Сканировать снова</button>
      </div>
    </template>
  </section>
</template>

<style scoped>
video {
  width: 100%;
  max-width: 480px;
  border-radius: 8px;
  background: #000;
  min-height: 240px;
}
.hidden {
  display: none;
}
.scan-box {
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: flex-start;
}
.mono {
  font-family: monospace;
  word-break: break-all;
}
.result {
  margin-top: 14px;
  max-width: 480px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.warn {
  color: #ff6b6b;
}
.ok {
  color: var(--accent);
}
.ghost {
  margin-top: 6px;
}
</style>
