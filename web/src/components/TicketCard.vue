<script setup>
import { computed, onMounted, ref } from 'vue'
import QRCode from 'qrcode'
import { fmtDate } from '../api'

const props = defineProps({
  ticket: { type: Object, required: true },
})

const qrData = ref('')

const statusText = computed(() =>
  props.ticket.status === 'used' ? 'использован' : 'действителен',
)

onMounted(async () => {
  qrData.value = await QRCode.toDataURL(props.ticket.mint, {
    margin: 1,
    width: 220,
    color: { dark: '#0d1117', light: '#ffffff' },
  })
})
</script>

<template>
  <div class="card ticket" :class="{ used: ticket.status === 'used' }">
    <div class="info">
      <h3>{{ ticket.event_title }}</h3>
      <p class="muted">{{ fmtDate(ticket.event_starts_at) }} · {{ ticket.event_city }}</p>
      <p>
        Статус:
        <strong :class="ticket.status">{{ statusText }}</strong>
      </p>
      <p v-if="ticket.status === 'used'" class="muted">
        вход отмечен {{ fmtDate(ticket.checked_in_at) }}
      </p>
      <p class="mono muted mint">{{ ticket.mint }}</p>
    </div>
    <img v-if="qrData" :src="qrData" alt="QR" class="qr" />
  </div>
</template>

<style scoped>
.ticket {
  display: flex;
  justify-content: space-between;
  gap: 16px;
}
.ticket.used {
  opacity: 0.7;
}
.info {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
h3 {
  margin: 0;
  font-size: 17px;
}
.mint {
  font-size: 11px;
  word-break: break-all;
}
.mono {
  font-family: monospace;
}
.qr {
  width: 150px;
  height: 150px;
  border-radius: 8px;
  align-self: center;
}
.valid {
  color: var(--accent);
}
.used {
  color: #ff6b6b;
}
</style>
