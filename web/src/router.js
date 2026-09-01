import { createRouter, createWebHistory } from 'vue-router'
import EventsView from './views/EventsView.vue'
import EventView from './views/EventView.vue'
import MyTicketsView from './views/MyTicketsView.vue'
import OrganizerView from './views/OrganizerView.vue'
import ScannerView from './views/ScannerView.vue'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'events', component: EventsView },
    { path: '/event/:pubkey', name: 'event', component: EventView, props: true },
    { path: '/tickets', name: 'my-tickets', component: MyTicketsView },
    { path: '/organizer', name: 'organizer', component: OrganizerView },
    { path: '/scan', name: 'scan', component: ScannerView },
  ],
})
