<script setup lang="ts">
import { onMounted } from 'vue'
import BorrowedItemsList from '../components/mypage/BorrowedItemsList.vue'
import PasswordChangeForm from '../components/mypage/PasswordChangeForm.vue'
import { useAppStore } from '../stores/counter'

const store = useAppStore()

onMounted(async () => {
  await store.getCurrentUser()
})

const onPasswordChangeSuccess = () => {
  // パスワード変更成功時の処理
  alert('パスワードが正常に変更されました。')
}
</script>

<template>
  <div :class="$style.container">
    <div :class="$style.header">
      <h1>マイページ</h1>
    </div>

    <div v-if="store.error" :class="$style.error">
      {{ store.error }}
    </div>

    <div :class="$style.content">
      <!-- ユーザー情報セクション -->
      <div :class="$style.section">
        <h2 :class="$style.sectionTitle">ユーザー情報</h2>
        <div v-if="store.currentUser" :class="$style.userInfo">
          <div :class="$style.infoItem">
            <span :class="$style.label">名前:</span>
            <span :class="$style.value">{{ store.currentUser.name }}</span>
          </div>
          <div :class="$style.infoItem">
            <span :class="$style.label">メールアドレス:</span>
            <span :class="$style.value">{{ store.currentUser.email }}</span>
          </div>
          <div :class="$style.infoItem">
            <span :class="$style.label">ロール:</span>
            <span :class="$style.value">{{ store.currentUser.role }}</span>
          </div>
        </div>
      </div>

      <!-- パスワード変更セクション -->
      <div :class="$style.section">
        <h2 :class="$style.sectionTitle">パスワード変更</h2>
        <PasswordChangeForm @success="onPasswordChangeSuccess" />
      </div>

      <!-- 借用中アイテム一覧セクション -->
      <div :class="$style.section">
        <h2 :class="$style.sectionTitle">借用中の物品</h2>
        <BorrowedItemsList />
      </div>
    </div>
  </div>
</template>

<style module>
.container {
  padding: 20px;
  width: 90%;
  max-width: 1000px;
  margin: 0 auto;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 50px;
  margin-bottom: 30px;
  border-bottom: 2px solid var(--color-accent);
  padding-bottom: 15px;
}

.header h1 {
  color: var(--color-text);
  margin: 0;
  font-size: 28px;
  font-weight: 600;
}

.error {
  background: color-mix(in srgb, var(--color-error) 10%, var(--color-background));
  color: var(--color-error);
  padding: 12px;
  border-radius: 8px;
  margin-bottom: 20px;
}

.content {
  display: flex;
  flex-direction: column;
  gap: 40px;
}

.section {
  background: var(--color-background);
  border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 2px 8px color-mix(in srgb, var(--color-accent) 15%, transparent);
}

.sectionTitle {
  color: var(--color-text);
  font-size: 20px;
  font-weight: 600;
  margin: 0 0 20px 0;
  padding-bottom: 10px;
  border-bottom: 1px solid color-mix(in srgb, var(--color-accent) 20%, transparent);
}

.userInfo {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.infoItem {
  display: flex;
  align-items: center;
  gap: 12px;
}

.label {
  font-weight: 500;
  color: var(--color-text);
  min-width: 120px;
}

.value {
  color: color-mix(in srgb, var(--color-text) 80%, transparent);
  padding: 8px 12px;
  background: color-mix(in srgb, var(--color-accent) 8%, transparent);
  border-radius: 6px;
  border: 1px solid color-mix(in srgb, var(--color-accent) 20%, transparent);
}

@media (max-width: 768px) {
  .container {
    width: 95%;
    padding: 15px;
  }

  .infoItem {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }

  .label {
    min-width: auto;
  }

  .value {
    width: 100%;
  }
}
</style>
