import { createApp } from 'vue'
import { initWalletStore } from './wallet/store'
import App from './App.vue'
import { router } from './router'
import { buildWalletAdapters } from './wallets'
import './style.css'

initWalletStore(buildWalletAdapters(), (error) => {
  console.warn('wallet error', error)
})

createApp(App).use(router).mount('#app')
