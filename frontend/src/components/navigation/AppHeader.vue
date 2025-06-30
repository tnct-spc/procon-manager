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
  height: 100px;
  width: 100%;
  background: var(--color-accent);
  color: var(--color-background);
  padding: 0 24px;
  box-shadow: 0 2px 8px color-mix(in srgb, var(--color-accent) 30%, transparent);
}

.brand h1 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--color-background);
}

.nav {
  display: flex;
  gap: 24px;
}

.navLink {
  color: color-mix(in srgb, var(--color-background) 90%, transparent);
  text-decoration: none;
  padding: 8px 16px;
  border-radius: 4px;
  transition: all 0.2s;
  font-weight: 500;
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
  gap: 24px;
}

.userInfo {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  font-size: 14px;
  color: color-mix(in srgb, var(--color-background) 90%, transparent);
  line-height: 1.2;
}

.userName {
  font-weight: 500;
}

.role {
  font-size: 12px;
  opacity: 0.8;
}

.logoutBtn {
  background: color-mix(in srgb, var(--color-background) 10%, transparent);
  color: var(--color-background);
  border: 1px solid color-mix(in srgb, var(--color-background) 30%, transparent);
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.logoutBtn:hover {
  background: color-mix(in srgb, var(--color-background) 20%, transparent);
  border-color: color-mix(in srgb, var(--color-background) 50%, transparent);
}
</style>
