<script setup>
import { lamportsToSol, fmtDate } from '../api'

const props = defineProps({
  event: { type: Object, required: true },
  now: { type: Number, required: true },
})
</script>

<template>
  <RouterLink :to="`/event/${event.pubkey}`" class="card event-card">
    <div class="row">
      <h3>{{ event.title }}</h3>
      <span v-if="event.hot_sale" class="badge">очередь</span>
    </div>
    <p class="muted">{{ fmtDate(event.starts_at) }} · {{ event.city }} · {{ event.venue }}</p>
    <div class="row footer">
      <span class="price">{{ lamportsToSol(event.ticket_price_lamports) }} SOL</span>
      <span class="muted">осталось {{ event.tickets_left }} из {{ event.capacity }}</span>
    </div>
  </RouterLink>
</template>

<style scoped>
.event-card {
  display: block;
  text-decoration: none;
  color: var(--text);
}
.event-card:hover {
  border-color: var(--accent);
}
.row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}
h3 {
  margin: 0 0 6px;
  font-size: 17px;
}
.footer {
  margin-top: 14px;
}
.price {
  color: var(--accent);
  font-weight: 600;
}
.badge {
  font-size: 11px;
  border: 1px solid var(--accent2);
  color: var(--accent2);
  border-radius: 999px;
  padding: 2px 8px;
  white-space: nowrap;
}
</style>
