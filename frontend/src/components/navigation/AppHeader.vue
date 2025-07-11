<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useAppStore } from '../../stores/counter'

const router = useRouter()
const store = useAppStore()

const handleLogout = () => {
  localStorage.removeItem('accessToken')
  localStorage.removeItem('userId')
  store.currentUser = null
  router.push('/login')
}
</script>

<template>
  <div :class="$style.navbar">
    <div :class="$style.brand">
      <h1>Procon Manager</h1>
    </div>

    <div v-if="store.currentUser" :class="$style.userSection">
      <nav :class="$style.nav">
        <router-link to="/dashboard" :class="$style.navLink" active-class="active">
          物品一覧
        </router-link>
        <router-link to="/mypage" :class="$style.navLink" active-class="active">
          マイページ
        </router-link>
        <router-link
          v-if="store.currentUser?.role === 'Admin'"
          to="/admin"
          :class="$style.navLink"
          active-class="active"
        >
          管理者操作
        </router-link>
      </nav>

      <div :class="$style.userInfo">
        <span :class="$style.userName">{{ store.currentUser.name }}</span>
        <span :class="$style.role">({{ store.currentUser.role }})</span>
      </div>

      <button @click="handleLogout" :class="$style.logoutBtn">ログアウト</button>
    </div>
  </div>
</template>

<style module>
.navbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: clamp(60px, 10vh, 100px);
  width: 100%;
  background: var(--color-accent);
  color: var(--color-background);
  padding: clamp(0.75rem, 3vw, 1.5rem);
  box-shadow: 0 2px 8px color-mix(in srgb, var(--color-accent) 30%, transparent);
  flex-wrap: wrap;
  gap: clamp(1rem, 2vw, 2rem);
}

.brand h1 {
  margin: 0;
  font-size: clamp(1.125rem, 4vw, 1.5rem);
  font-weight: 600;
  color: var(--color-background);
}

.nav {
  display: flex;
  gap: clamp(0.75rem, 2vw, 1.5rem);
  flex-wrap: wrap;
  justify-content: center;
}

.navLink {
  color: color-mix(in srgb, var(--color-background) 90%, transparent);
  text-decoration: none;
  padding: clamp(0.25rem, 1vw, 0.5rem) clamp(0.5rem, 2vw, 1rem);
  border-radius: 4px;
  transition: all 0.2s;
  font-weight: 500;
  font-size: clamp(0.75rem, 2vw, 1rem);
  white-space: nowrap;
}

.navLink:hover {
  background: color-mix(in srgb, var(--color-background) 10%, transparent);
  color: var(--color-background);
}

.navLink.active {
  background: color-mix(in srgb, var(--color-background) 20%, transparent);
  color: var(--color-background);
}

.userSection {
  display: flex;
  align-items: center;
  gap: clamp(0.75rem, 2vw, 1.5rem);
  flex-wrap: wrap;
  justify-content: center;
}

.userInfo {
  display: flex;
  flex-direction: column;
  align-items: center;
  font-size: clamp(0.75rem, 2vw, 0.875rem);
  color: color-mix(in srgb, var(--color-background) 90%, transparent);
  line-height: 1.2;
  text-align: center;
}

.userName {
  font-weight: 500;
}

.role {
  font-size: clamp(0.625rem, 1.5vw, 0.75rem);
  opacity: 0.8;
}

.logoutBtn {
  background: color-mix(in srgb, var(--color-background) 10%, transparent);
  color: var(--color-background);
  border: 1px solid color-mix(in srgb, var(--color-background) 30%, transparent);
  padding: clamp(0.375rem, 1vw, 0.5rem) clamp(0.75rem, 2vw, 1rem);
  border-radius: 4px;
  cursor: pointer;
  font-size: clamp(0.75rem, 2vw, 0.875rem);
  transition: all 0.2s;
  white-space: nowrap;
}

.logoutBtn:hover {
  background: color-mix(in srgb, var(--color-background) 20%, transparent);
  border-color: color-mix(in srgb, var(--color-background) 50%, transparent);
}

@container (max-width: 50rem) {
  .navbar {
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .userSection {
    flex-direction: column;
    width: 100%;
    align-items: center;
  }
}
</style>
