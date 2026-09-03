import { createApp } from 'vue'
import { initWallet } from '@solana/wallet-adapter-vue'
import App from './App.vue'
import { router } from './router'
import { buildWalletAdapters } from './wallets'
import './style.css'

initWallet({
  wallets: buildWalletAdapters(),
  autoConnect: true,
  onError(error: unknown) {
    console.warn('wallet error', error)
  },
})

createApp(App).use(router).mount('#app')
