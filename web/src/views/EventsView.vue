<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed } from 'vue'
import { api, type EventSummary } from '../api'
import EventCard from '../components/EventCard.vue'

const events = ref<EventSummary[]>([])
const total = ref(0)
const error = ref<string | null>(null)
const loading = ref(true)
const city = ref('')
const query = ref('')
const upcoming = ref(true)
const now = ref(Math.floor(Date.now() / 1000))

let timer: ReturnType<typeof setInterval> | undefined

async function load() {
  loading.value = true
  error.value = null
  try {
    const data = await api.events({
      city: city.value,
      q: query.value,
      upcoming: upcoming.value,
    })
    events.value = data.events || []
    total.value = data.total || 0
  } catch (e) {
    error.value = 'Не удалось загрузить афишу'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  load()
  timer = setInterval(() => (now.value = Math.floor(Date.now() / 1000)), 1000)
})
onUnmounted(() => clearInterval(timer))

const filtered = computed(() =>
  events.value.filter((e) => e.starts_at > now.value || !upcoming.value),
)
</script>

<template>
  <section>
    <div class="toolbar">
      <input v-model="city" placeholder="Город" @change="load" />
      <input v-model="query" placeholder="Поиск" @change="load" />
      <label class="muted">
        <input type="checkbox" v-model="upcoming" @change="load" />
        Только предстоящие
      </label>
    </div>

    <p v-if="error" class="muted">{{ error }}</p>
    <p v-else-if="loading" class="muted">Загрузка…</p>
    <p v-else-if="filtered.length === 0" class="muted">
      Событий нет. Афиша живёт ончейн — создайте событие, чтобы оно появилось здесь.
    </p>

    <div class="grid">
      <EventCard v-for="e in filtered" :key="e.pubkey" :event="e" :now="now" />
    </div>
  </section>
</template>

<style scoped>
.toolbar {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
  flex-wrap: wrap;
  align-items: center;
}
input {
  background: var(--surface);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 8px;
  padding: 8px 12px;
  font: inherit;
}
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}
</style>
