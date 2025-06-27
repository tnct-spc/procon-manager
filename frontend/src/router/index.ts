import { createRouter, createWebHistory } from 'vue-router'
import LoginForm from '../components/login/LoginForm.vue'
import ItemDashboard from '../components/dashboard/ItemDashboard.vue'
import AdminView from '../views/AdminView.vue'
import MyPageView from '../views/MyPageView.vue'
import { useAppStore } from '../stores/counter'

const routes = [
  { path: '/', redirect: '/login' },
  { path: '/login', name: 'Login', component: LoginForm },
  { 
    path: '/dashboard', 
    name: 'Dashboard', 
    component: ItemDashboard,
    meta: { requiresAuth: true }
  },
  { 
    path: '/admin', 
    name: 'Admin', 
    component: AdminView,
    meta: { requiresAuth: true, requiresAdmin: true }
  },
  { 
    path: '/mypage', 
    name: 'MyPage', 
    component: MyPageView,
    meta: { requiresAuth: true }
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// ナビゲーションガード追加
router.beforeEach(async (to) => {
  const store = useAppStore()
  if (store.currentUser === null) {
    await store.getCurrentUser()
  }
  const requiresAuth = to.matched.some(r => r.meta.requiresAuth)
  const requiresAdmin = to.matched.some(r => r.meta.requiresAdmin)
  
  if (requiresAuth && !store.currentUser) {
    return { name: 'Login' }
  }
  
  if (requiresAdmin && store.currentUser?.role !== 'Admin') {
    return { name: 'Dashboard' }
  }
  
  if (to.name === 'Login' && store.currentUser) {
    return { name: 'Dashboard' }
  }
})

export default router
