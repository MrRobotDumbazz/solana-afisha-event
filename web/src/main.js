import { createApp } from 'vue'
import { initWallet } from '@solana/wallet-adapter-vue'
import App from './App.vue'
import { router } from './router'
import './style.css'

initWallet({
  wallets: [],
  autoConnect: true,
})

createApp(App).use(router).mount('#app')
