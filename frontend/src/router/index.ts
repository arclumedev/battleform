import { createRouter, createWebHistory } from 'vue-router'
const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', redirect: '/lobby' },
    { path: '/login', name: 'login', component: () => import('../views/LoginView.vue') },
    { path: '/lobby', name: 'lobby', component: () => import('../views/LobbyView.vue') },
    { path: '/match/:id', name: 'match', component: () => import('../views/MatchView.vue') },
  ],
})
export default router
