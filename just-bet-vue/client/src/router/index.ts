import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '~stores/useAuthStore'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: () => import('~pages/LoginView.vue'),
      meta: { requiresAuth: false, navigational: false }
    },
    {
      path: '/register',
      name: 'register',
      component: () => import('~pages/RegisterView.vue'),
      meta: { requiresAuth: false, navigational: false }
    },
    {
      path: '/minesweeper',
      name: 'minesweeper',
      component: () => import('~pages/MinesweeperView.vue'),
      meta: { requiresAuth: true, navigational: true, navigateTo: true }
    },
    {
      path: '/battleships',
      name: 'battleships',
      component: () => import('~pages/BattleshipsView.vue'),
      meta: { requiresAuth: true, navigational: true, navigateTo: true }
    },
    {
      path: '/battleships/:gameid',
      name: 'battleships game',
      component: () => import('~pages/BattleshipsGameView.vue'),
      meta: { requiresAuth: true, navigational: true }
    },
    {
      path: '/game-history',
      name: 'history',
      component: () => import('~pages/GameHistoryView.vue'),
      meta: { requiresAuth: true, navigational: true, navigateTo: true }
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: '/minesweeper'
    }
  ]
})

router.beforeEach(async (to, from) => {
  const authStore = useAuthStore()
  if (to.meta.requiresAuth) {
    if (!(await authStore.isAuthenticated())) return { name: 'login' }
    return true
  }
  return true
})

export default router
