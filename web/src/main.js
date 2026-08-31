import { createApp } from 'vue'
import { initWallet } from '@solana/wallet-adapter-vue'
import App from './App.vue'
import './style.css'

initWallet({
  wallets: [],
  autoConnect: true,
})

createApp(App).mount('#app')
