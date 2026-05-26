import { createRouter, createWebHistory } from 'vue-router'
import LandingView from '@/views/LandingView.vue'
import LoginView from '@/views/LoginView.vue'
import AppView from '@/views/AppView.vue'
import ProfileView from '@/views/ProfileView.vue'
import ThreadView from '@/views/ThreadView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'landing',
      component: LandingView,
    },
    {
      path: '/login',
      name: 'login',
      component: LoginView,
    },
    {
      path: '/app',
      name: 'app',
      component: AppView,
    },
    {
      path: '/profile',
      name: 'profile',
      component: ProfileView,
    },
    {
      path: '/thread',
      name: 'thread',
      component: ThreadView,
    },
  ],
})

export default router
