<script setup>
import { computed, ref } from 'vue'
import { useWallet } from '@solana/wallet-adapter-vue'

const { wallets, select, disconnect, publicKey, connecting } = useWallet()
const open = ref(false)

const list = computed(() =>
  wallets.value.map((w) => ({
    name: w.adapter.name,
    icon: w.adapter.icon,
    readyState: w.readyState,
    installed: w.readyState === 'Installed',
  })),
)

function pick(name) {
  select(name)
  open.value = false
}
</script>

<template>
  <div class="connect">
    <button v-if="!publicKey" class="primary" @click="open = true">
      Подключить кошелёк
    </button>
    <button v-else @click="disconnect()">Отключить</button>

    <div v-if="open" class="overlay" @click.self="open = false">
      <div class="modal card">
        <h3>Выберите кошелёк</h3>
        <div class="wallets">
          <button
            v-for="w in list"
            :key="w.name"
            class="wallet"
            :class="{ dim: !w.installed }"
            :disabled="connecting"
            @click="pick(w.name)"
          >
            <img v-if="w.icon" :src="w.icon" :alt="w.name" />
            <span class="name">{{ w.name }}</span>
            <span class="state muted">{{ w.installed ? '' : 'не установлен' }}</span>
          </button>
        </div>
        <p class="muted hint">
          Phantom, Solflare, Backpack, Glow, Brave, Exodus и другие Wallet Standard
          кошельки находятся автоматически при установке расширения.
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.connect {
  display: inline-flex;
}
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
}
.modal {
  width: min(420px, 92vw);
  max-height: 80vh;
  overflow: auto;
}
h3 {
  margin: 0 0 12px;
}
.wallets {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.wallet {
  display: grid;
  grid-template-columns: 32px 1fr auto;
  align-items: center;
  gap: 12px;
  text-align: left;
  padding: 10px 12px;
}
.wallet:hover {
  border-color: var(--accent);
}
.wallet img {
  width: 32px;
  height: 32px;
  border-radius: 6px;
}
.dim {
  opacity: 0.55;
}
.state {
  font-size: 11px;
}
.hint {
  font-size: 12px;
  margin: 12px 0 0;
}
</style>
