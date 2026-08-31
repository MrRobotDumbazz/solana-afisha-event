<script setup>
import { ref, onMounted } from 'vue'

const events = ref(null)
const error = ref(null)

onMounted(async () => {
  try {
    const res = await fetch('/api/v1/events')
    const data = await res.json()
    events.value = data.events
  } catch (e) {
    error.value = 'Не удалось загрузить афишу'
  }
})
</script>

<template>
  <section>
    <h2>Афиша</h2>
    <p v-if="error" class="muted">{{ error }}</p>
    <p v-else-if="events === null" class="muted">Загрузка…</p>
    <p v-else-if="events.length === 0" class="muted">Событий пока нет — афиша пуста.</p>
    <div v-else class="grid">
      <article v-for="e in events" :key="e.pubkey" class="card">
        <h3>{{ e.title }}</h3>
      </article>
    </div>
  </section>
</template>
