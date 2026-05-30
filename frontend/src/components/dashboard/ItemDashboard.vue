<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useAppStore } from '../../stores/counter'
import type { Item } from '../../types/api'
import { getErrorMessage, type ApiError } from '../../types/error'
import AddButton from '../ui/AddButton.vue'
import CreateItemForm from './CreateItemForm.vue'
import EditItemForm from './EditItemForm.vue'

const store = useAppStore()
const showCreateForm = ref(false)
const isAdmin = computed(() => store.currentUser?.role === 'Admin')
const pendingCheckoutItemId = ref<string | null>(null)
const pendingReturnItemId = ref<string | null>(null)
const selectedCheckoutUserId = ref<string>('')
const checkoutUserQuery = ref('')
const itemSearchQuery = ref('')

const formatCheckoutUser = (user: { name: string; email: string }) => `${user.name} (${user.email})`

const selectedCheckoutUser = computed(() =>
  store.users.find((user) => user.id === selectedCheckoutUserId.value),
)

const pendingCheckoutItem = computed(() =>
  store.items.find((item) => item.id === pendingCheckoutItemId.value),
)

const pendingReturnItem = computed(() =>
  store.items.find((item) => item.id === pendingReturnItemId.value),
)

const hasPendingAction = computed(() => !!pendingCheckoutItem.value || !!pendingReturnItem.value)

const getItemSearchText = (item: Item) =>
  [
    item.name,
    item.description,
    item.location,
    getItemTypeLabel(item),
    getItemDetails(item),
    item.checkout?.checkedOutBy.name,
  ]
    .filter(Boolean)
    .join(' ')
    .toLowerCase()

const filteredItems = computed(() => {
  const query = itemSearchQuery.value.trim().toLowerCase()
  if (!query) return store.items

  return store.items.filter((item) => getItemSearchText(item).includes(query))
})

const filteredCheckoutUsers = computed(() => {
  const query = checkoutUserQuery.value.trim().toLowerCase()
  if (!query) return store.users

  return store.users.filter((user) => formatCheckoutUser(user).toLowerCase().includes(query))
})

const handleCheckout = async (item: Item) => {
  if (!store.currentUser) {
    alert('ログインが必要です。')
    return
  }

  pendingCheckoutItemId.value = item.id
  pendingReturnItemId.value = null
  selectedCheckoutUserId.value = store.currentUser.id
  checkoutUserQuery.value = ''
}

const cancelCheckout = () => {
  pendingCheckoutItemId.value = null
  selectedCheckoutUserId.value = ''
  checkoutUserQuery.value = ''
}

const cancelReturn = () => {
  pendingReturnItemId.value = null
}

const selectCheckoutUser = (userId: string) => {
  const user = store.users.find((checkoutUser) => checkoutUser.id === userId)
  if (!user) return

  selectedCheckoutUserId.value = user.id
  checkoutUserQuery.value = ''
}

const confirmCheckout = async (item: Item) => {
  if (!store.currentUser) {
    alert('ログインが必要です。')
    return
  }

  const checkedOutBy = isAdmin.value ? selectedCheckoutUserId.value : store.currentUser.id
  if (!checkedOutBy) {
    alert('借りるユーザーを検索して選択してください。')
    return
  }

  try {
    await store.checkoutItem(item.id, checkedOutBy)
    cancelCheckout()
  } catch (error: unknown) {
    console.error('チェックアウトエラー:', error)
    const apiError = error as ApiError
    if (apiError.response?.status === 404) {
      alert('アイテムが見つかりません。')
    } else if (apiError.response?.status === 409) {
      alert('このアイテムは既にチェックアウトされています。')
    } else if (apiError.response?.status === 401) {
      alert('ログインが必要です。')
    } else if (apiError.response?.status === 403) {
      alert('指定したユーザーで借りる権限がありません。')
    } else {
      alert(`チェックアウトに失敗しました: ${getErrorMessage(error)}`)
    }
  }
}

const handleReturn = (item: Item) => {
  if (!item.checkout) return

  pendingReturnItemId.value = item.id
  cancelCheckout()
}

const confirmReturn = async (item: Item) => {
  if (!item.checkout) return

  try {
    await store.returnItem(item.id, item.checkout.id)
    cancelReturn()
  } catch (error: unknown) {
    console.error('返却エラー:', error)
    const apiError = error as ApiError
    if (apiError.response?.status === 404) {
      alert('アイテムまたはチェックアウトが見つかりません。')
    } else if (apiError.response?.status === 403) {
      alert('返却する権限がありません。')
    } else if (apiError.response?.status === 401) {
      alert('ログインが必要です。')
    } else {
      alert(`返却に失敗しました: ${getErrorMessage(error)}`)
    }
  }
}

const getItemTypeLabel = (item: Item) => {
  switch (item.category) {
    case 'general':
      return '一般'
    case 'book':
      return '書籍'
    case 'laptop':
      return 'ノートPC'
    default:
      return '不明'
  }
}

const getItemDetails = (item: Item) => {
  switch (item.category) {
    case 'book':
      return `著者: ${item.author}, ISBN: ${item.isbn}`
    case 'laptop':
      return `MAC: ${item.macAddress}`
    default:
      return ''
  }
}

const showEditForm = ref(false)
const editingItem = ref<Item | null>(null)
const showMenu = ref<{ [key: string]: boolean }>({})

const toggleMenu = (itemId: string) => {
  const isCurrentlyOpen = showMenu.value[itemId]
  showMenu.value = {
    [itemId]: !isCurrentlyOpen,
  }
}

const editItem = (item: Item) => {
  if (!isAdmin.value) return

  editingItem.value = item
  showEditForm.value = true
  showMenu.value = {}
}

const deleteItem = async (itemId: string) => {
  if (!isAdmin.value) return

  if (!confirm('このアイテムを削除しますか？この操作は取り消せません。')) return

  try {
    await store.deleteItem(itemId)
  } catch (error: unknown) {
    console.error('削除エラー:', error)
    const apiError = error as ApiError
    alert(`削除に失敗しました: ${apiError.message || 'サーバーエラー'}`)
  }
  showMenu.value = {}
}

const closeAllMenus = () => {
  showMenu.value = {}
}

const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as Element
  if (!target.closest('[data-menu-container]')) {
    closeAllMenus()
  }
}

onMounted(async () => {
  await store.getCurrentUser()
  await store.fetchItems()
  if (store.currentUser?.role === 'Admin') {
    try {
      await store.fetchUsers()
    } catch (error: unknown) {
      console.error('ユーザー一覧の取得に失敗:', error)
    }
  }
  document.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div :class="$style.container">
    <div :class="$style.header">
      <h1>アイテム管理</h1>
    </div>

    <div :class="$style.searchBar">
      <input
        v-model="itemSearchQuery"
        :class="$style.searchInput"
        type="search"
        placeholder="アイテム名、説明、場所、種別、貸出ユーザーで検索"
        aria-label="アイテム検索"
      />
      <span :class="$style.searchCount">
        {{ filteredItems.length }} / {{ store.items.length }} 件
      </span>
    </div>

    <div v-if="store.error" :class="$style.error">
      {{ store.error }}
    </div>

    <div v-if="store.loading" :class="$style.loading">読み込み中...</div>

    <div v-else :class="$style.itemList">
      <div
        v-for="item in filteredItems"
        :key="item.id"
        :class="[$style.itemCard, { [$style.checkedOut]: item.checkout }]"
      >
        <span :class="$style.itemType">{{ getItemTypeLabel(item) }}</span>
        <div :class="$style.itemInfo">
          <div :class="$style.itemHeader">
            <h3 :class="$style.itemName">{{ item.name }}</h3>
          </div>

          <p :class="$style.description">{{ item.description }}</p>

          <div v-if="item.location" :class="$style.location">場所: {{ item.location }}</div>

          <div v-if="getItemDetails(item)" :class="$style.details">
            {{ getItemDetails(item) }}
          </div>

          <div v-if="item.checkout" :class="$style.checkoutInfo">
            <span :class="$style.checkoutLabel">チェックアウト中:</span>
            <span :class="$style.checkoutUser">{{ item.checkout.checkedOutBy.name }}</span>
            <span :class="$style.checkoutDate"
              >({{ new Date(item.checkout.checkedOutAt).toLocaleDateString('ja-JP') }})</span
            >
          </div>
        </div>

        <div :class="$style.itemActions">
          <div :class="$style.primaryActions">
            <button
              v-if="!item.checkout"
              @click="handleCheckout(item)"
              :class="[
                $style.checkoutBtn,
                { [$style.checkoutBtnActive]: pendingCheckoutItemId === item.id },
              ]"
              :disabled="store.loading || pendingCheckoutItemId === item.id"
            >
              {{ pendingCheckoutItemId === item.id ? '確認中' : '借りる' }}
            </button>

            <button
              v-if="
                item.checkout &&
                store.currentUser &&
                (store.currentUser.role === 'Admin' ||
                  item.checkout.checkedOutBy.id === store.currentUser.id)
              "
              @click="handleReturn(item)"
              :class="[
                $style.returnBtn,
                { [$style.returnBtnActive]: pendingReturnItemId === item.id },
              ]"
              :disabled="store.loading || pendingReturnItemId === item.id"
            >
              {{ pendingReturnItemId === item.id ? '確認中' : '返却' }}
            </button>
          </div>

          <div
            v-if="isAdmin && !hasPendingAction"
            :class="$style.menuContainer"
            data-menu-container
          >
            <button
              @click="toggleMenu(item.id)"
              :class="$style.menuBtn"
              :aria-label="'メニューを開く'"
            >
              ⋮
            </button>

            <div v-if="showMenu[item.id]" :class="$style.dropdown">
              <button @click="editItem(item)" :class="$style.dropdownItem">編集</button>
              <button
                @click="deleteItem(item.id)"
                :class="[$style.dropdownItem, $style.deleteAction]"
              >
                削除
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="pendingCheckoutItem" :class="$style.checkoutOverlay" @click.self="cancelCheckout">
      <section :class="$style.checkoutConfirm" aria-label="借用内容">
        <div :class="$style.checkoutConfirmHeader">
          <div>
            <span :class="$style.checkoutConfirmTitle">借用内容</span>
            <p :class="$style.checkoutItemName">{{ pendingCheckoutItem.name }}</p>
          </div>
          <button
            @click="cancelCheckout"
            :class="$style.inlineCloseBtn"
            :disabled="store.loading"
            aria-label="借用内容の確認を閉じる"
          >
            ×
          </button>
        </div>

        <div v-if="isAdmin" :class="$style.checkoutField">
          <span :class="$style.checkoutFieldLabel">借りるユーザー</span>
          <div v-if="selectedCheckoutUser" :class="$style.selectedCheckoutUser">
            <span :class="$style.selectedCheckoutUserLabel">選択中</span>
            <span :class="$style.selectedCheckoutUserName">{{ selectedCheckoutUser.name }}</span>
            <span :class="$style.selectedCheckoutUserEmail">{{ selectedCheckoutUser.email }}</span>
          </div>
          <input
            v-model="checkoutUserQuery"
            :class="$style.checkoutSearch"
            :disabled="store.loading"
            type="search"
            placeholder="名前またはメールで検索"
            aria-label="借りるユーザー"
            @input="selectedCheckoutUserId = ''"
          />
          <div :class="$style.checkoutUserList">
            <button
              v-for="checkoutUser in filteredCheckoutUsers"
              :key="checkoutUser.id"
              type="button"
              :class="[
                $style.checkoutUserOption,
                {
                  [$style.checkoutUserOptionSelected]: checkoutUser.id === selectedCheckoutUserId,
                },
              ]"
              :disabled="store.loading"
              @click="selectCheckoutUser(checkoutUser.id)"
            >
              <span :class="$style.checkoutUserName">{{ checkoutUser.name }}</span>
              <span :class="$style.checkoutUserEmail">{{ checkoutUser.email }}</span>
            </button>
            <span v-if="filteredCheckoutUsers.length === 0" :class="$style.noCheckoutUsers">
              該当するユーザーがいません
            </span>
          </div>
        </div>

        <div v-else :class="$style.checkoutSummary">
          {{ store.currentUser?.name }} として借ります
        </div>

        <div :class="$style.checkoutConfirmActions">
          <button
            @click="confirmCheckout(pendingCheckoutItem)"
            :class="$style.confirmCheckoutBtn"
            :disabled="store.loading || (isAdmin && !selectedCheckoutUserId)"
          >
            確定
          </button>
          <button
            @click="cancelCheckout"
            :class="$style.cancelCheckoutBtn"
            :disabled="store.loading"
          >
            キャンセル
          </button>
        </div>
      </section>
    </div>

    <div v-if="pendingReturnItem" :class="$style.checkoutOverlay" @click.self="cancelReturn">
      <section :class="$style.checkoutConfirm" aria-label="返却内容">
        <div :class="$style.checkoutConfirmHeader">
          <div>
            <span :class="$style.checkoutConfirmTitle">返却内容</span>
            <p :class="$style.checkoutItemName">{{ pendingReturnItem.name }}</p>
          </div>
          <button
            @click="cancelReturn"
            :class="$style.inlineCloseBtn"
            :disabled="store.loading"
            aria-label="返却内容の確認を閉じる"
          >
            ×
          </button>
        </div>

        <div v-if="pendingReturnItem.checkout" :class="$style.returnSummary">
          <span :class="$style.returnSummaryLabel">借用者</span>
          <span :class="$style.returnSummaryName">
            {{ pendingReturnItem.checkout.checkedOutBy.name }}
          </span>
          <span :class="$style.returnSummaryDate">
            {{ new Date(pendingReturnItem.checkout.checkedOutAt).toLocaleDateString('ja-JP') }}
            から貸出中
          </span>
        </div>

        <div :class="$style.checkoutConfirmActions">
          <button
            @click="confirmReturn(pendingReturnItem)"
            :class="$style.confirmCheckoutBtn"
            :disabled="store.loading"
          >
            確定
          </button>
          <button @click="cancelReturn" :class="$style.cancelCheckoutBtn" :disabled="store.loading">
            キャンセル
          </button>
        </div>
      </section>
    </div>

    <div v-if="filteredItems.length === 0 && !store.loading" :class="$style.empty">
      {{ store.items.length === 0 ? 'アイテムがありません' : '条件に一致するアイテムがありません' }}
    </div>

    <!-- Pagination Controls -->
    <div v-if="store.totalPages > 1" :class="$style.pagination">
      <button
        @click="store.fetchItems(store.currentPage - 1)"
        :disabled="store.currentPage <= 1 || store.loading"
        :class="[$style.paginationBtn, $style.prevBtn]"
      >
        前へ
      </button>

      <span :class="$style.pageInfo">
        {{ store.currentPage }} / {{ store.totalPages }} ページ (合計: {{ store.totalItems }} 件)
      </span>

      <button
        @click="store.fetchItems(store.currentPage + 1)"
        :disabled="store.currentPage >= store.totalPages || store.loading"
        :class="[$style.paginationBtn, $style.nextBtn]"
      >
        次へ
      </button>
    </div>

    <CreateItemForm v-if="showCreateForm" @close="showCreateForm = false" />

    <!-- Edit Item Form Modal -->
    <div v-if="showEditForm" :class="$style.modalOverlay" @click="showEditForm = false">
      <div :class="$style.modal" @click.stop>
        <div :class="$style.modalHeader">
          <h3 :class="$style.modalTitle">アイテムを編集</h3>
          <button @click="showEditForm = false" :class="$style.closeBtn">×</button>
        </div>
        <div :class="$style.modalContent">
          <EditItemForm
            v-if="editingItem"
            :item="editingItem"
            @close="((showEditForm = false), (editingItem = null))"
          />
        </div>
      </div>
    </div>

    <AddButton
      v-if="isAdmin && !hasPendingAction"
      @click="showCreateForm = true"
      :disabled="store.loading"
      label="新しいアイテムを追加"
    />
  </div>
</template>

<style module>
.container {
  padding: 24px;
  width: min(1180px, calc(100% - 48px));
  margin: 0 auto;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: clamp(2rem, 5vh, 3rem);
  margin-bottom: clamp(1rem, 3vh, 1.5rem);
  border-bottom: 2px solid var(--color-accent);
  padding-bottom: clamp(0.75rem, 2vh, 1rem);
  flex-wrap: wrap;
  gap: clamp(0.5rem, 2vw, 1rem);
}

.headerActions {
  display: flex;
  align-items: center;
  gap: 16px;
}

.createBtn {
  background: var(--color-accent);
  color: var(--color-background);
  border: none;
  padding: 10px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: background-color 0.2s;
}

.createBtn:hover {
  background: color-mix(in srgb, var(--color-accent) 80%, black);
}

.userInfo {
  font-size: 14px;
  color: color-mix(in srgb, var(--color-text) 60%, transparent);
}

.error {
  background: color-mix(in srgb, var(--color-error) 10%, var(--color-background));
  color: var(--color-error);
  padding: 12px;
  border-radius: 4px;
  margin-bottom: 20px;
}

.loading {
  text-align: center;
  padding: 40px;
  color: color-mix(in srgb, var(--color-text) 60%, transparent);
}

.searchBar {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 12px;
  margin-bottom: 18px;
  padding: 12px;
  border: 1px solid color-mix(in srgb, var(--color-text) 10%, transparent);
  border-radius: 8px;
  background: color-mix(in srgb, var(--color-background) 88%, white);
}

.searchInput {
  width: 100%;
  min-height: 42px;
  border: 1px solid color-mix(in srgb, var(--color-text) 14%, transparent);
  border-radius: 4px;
  background: var(--color-background);
  color: var(--color-text);
  font-size: 14px;
  padding: 8px 12px;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
}

.searchInput:focus {
  border-color: var(--color-accent);
  outline: none;
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-accent) 16%, transparent);
}

.searchCount {
  color: color-mix(in srgb, var(--color-text) 62%, transparent);
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
}

.itemList {
  display: grid;
  gap: 14px;
}

.itemCard {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 360px;
  gap: 28px;
  align-items: stretch;
  padding: 24px;
  border: 1px solid color-mix(in srgb, var(--color-text) 14%, transparent);
  border-radius: 8px;
  background: var(--color-background);
  box-shadow: 0 1px 2px color-mix(in srgb, var(--color-text) 8%, transparent);
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
  min-height: 150px;
  position: relative;
}

.itemCard:hover {
  border-color: color-mix(in srgb, var(--color-accent) 48%, transparent);
  box-shadow: 0 8px 24px color-mix(in srgb, var(--color-text) 8%, transparent);
}

.checkedOut {
  border-left: 4px solid var(--color-error);
}

.itemInfo {
  justify-self: stretch;
  min-width: 0;
  padding-top: 28px;
}

.itemHeader {
  margin-bottom: clamp(0.5rem, 2vw, 0.75rem);
}

.itemName {
  margin: 0;
  font-size: clamp(1.125rem, 3vw, 1.375rem);
  font-weight: 600;
  color: var(--color-text);
}

.itemType {
  position: absolute;
  top: 20px;
  left: 24px;
  background: color-mix(in srgb, var(--color-accent) 10%, var(--color-background));
  color: color-mix(in srgb, var(--color-accent) 86%, var(--color-text));
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
  white-space: nowrap;
  letter-spacing: 0;
}

.description {
  color: color-mix(in srgb, var(--color-text) 70%, transparent);
  margin: clamp(0.5rem, 2vw, 0.75rem) 0;
  line-height: 1.5;
  font-size: clamp(0.875rem, 2.5vw, 1rem);
}

.location,
.details {
  font-size: clamp(0.875rem, 2.5vw, 1rem);
  color: var(--color-text);
  margin: clamp(0.5rem, 2vw, 0.75rem) 0;
  padding: clamp(0.5rem, 2vw, 0.75rem) clamp(0.75rem, 2vw, 1rem);
  background: color-mix(in srgb, var(--color-accent) 8%, var(--color-background));
  border-radius: 6px;
  border-left: 3px solid var(--color-accent);
  font-weight: 500;
}

.checkoutInfo {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 6px 0;
  font-size: 14px;
}

.checkoutLabel {
  color: var(--color-warning);
  font-weight: 500;
}

.checkoutUser {
  color: var(--color-text);
  font-weight: 500;
}

.checkoutDate {
  color: color-mix(in srgb, var(--color-text) 60%, transparent);
}

.checkoutInfo p {
  margin: 4px 0;
}

.itemActions {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  width: 360px;
  min-width: 0;
  min-height: 44px;
  justify-content: flex-end;
  justify-self: end;
}

.primaryActions {
  display: flex;
  justify-content: flex-end;
  position: relative;
}

.checkoutBtn,
.returnBtn,
.confirmCheckoutBtn,
.cancelCheckoutBtn {
  padding: clamp(0.5rem, 2vw, 0.75rem) clamp(0.75rem, 3vw, 1rem);
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 700;
  transition:
    background-color 0.2s,
    border-color 0.2s,
    color 0.2s;
  white-space: nowrap;
  min-width: 104px;
}

.checkoutBtn {
  background: var(--color-accent);
  color: var(--color-background);
}

.checkoutBtn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-accent) 80%, black);
}

.returnBtn {
  background: var(--color-warning);
  color: var(--color-background);
}

.returnBtn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-warning) 80%, black);
}

.checkoutBtn:disabled,
.returnBtn:disabled,
.confirmCheckoutBtn:disabled,
.cancelCheckoutBtn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.checkoutBtnActive:disabled {
  background: color-mix(in srgb, var(--color-accent) 86%, var(--color-text));
  color: var(--color-background);
  opacity: 1;
}

.returnBtnActive:disabled {
  background: color-mix(in srgb, var(--color-warning) 86%, var(--color-text));
  color: var(--color-background);
  opacity: 1;
}

.checkoutOverlay {
  position: fixed;
  inset: 0;
  z-index: 900;
  display: flex;
  justify-content: flex-end;
  align-items: flex-start;
  padding: 118px max(24px, calc((100vw - 1180px) / 2 + 24px)) 24px 24px;
  background: color-mix(in srgb, var(--color-text) 10%, transparent);
}

.checkoutConfirm {
  width: min(420px, calc(100vw - 48px));
  max-height: calc(100vh - 142px);
  overflow: auto;
  padding: 18px;
  border: 1px solid color-mix(in srgb, var(--color-text) 14%, transparent);
  border-radius: 8px;
  background: var(--color-background);
  box-shadow: 0 18px 46px color-mix(in srgb, var(--color-text) 20%, transparent);
}

.checkoutConfirmHeader {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.checkoutConfirmTitle {
  color: var(--color-text);
  font-size: 16px;
  font-weight: 700;
}

.checkoutItemName {
  margin: 4px 0 0;
  color: color-mix(in srgb, var(--color-text) 64%, transparent);
  font-size: 13px;
}

.inlineCloseBtn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: 1px solid color-mix(in srgb, var(--color-text) 12%, transparent);
  border-radius: 4px;
  background: var(--color-background);
  color: color-mix(in srgb, var(--color-text) 62%, transparent);
  cursor: pointer;
  font-size: 18px;
  line-height: 1;
}

.inlineCloseBtn:hover:not(:disabled) {
  color: var(--color-text);
  border-color: color-mix(in srgb, var(--color-text) 28%, transparent);
}

.checkoutField {
  display: grid;
  gap: 8px;
}

.checkoutFieldLabel {
  color: color-mix(in srgb, var(--color-text) 70%, transparent);
  font-size: 13px;
  font-weight: 500;
}

.checkoutSearch {
  width: 100%;
  min-height: 42px;
  border: 1px solid color-mix(in srgb, var(--color-text) 18%, transparent);
  border-radius: 4px;
  background: var(--color-background);
  color: var(--color-text);
  font-size: 14px;
  padding: 8px 12px;
}

.checkoutSearch:focus {
  border-color: var(--color-accent);
  outline: none;
}

.checkoutUserList {
  display: grid;
  gap: 4px;
  max-height: 220px;
  overflow-y: auto;
  padding: 4px;
  border: 1px solid color-mix(in srgb, var(--color-text) 12%, transparent);
  border-radius: 6px;
  background: var(--color-background);
}

.checkoutUserOption {
  display: grid;
  gap: 2px;
  width: 100%;
  padding: 8px 10px;
  border: 1px solid transparent;
  border-radius: 4px;
  background: transparent;
  color: var(--color-text);
  cursor: pointer;
  text-align: left;
}

.checkoutUserOption:hover:not(:disabled),
.checkoutUserOptionSelected {
  border-color: color-mix(in srgb, var(--color-accent) 36%, transparent);
  background: color-mix(in srgb, var(--color-accent) 10%, var(--color-background));
}

.checkoutUserName {
  font-size: 14px;
  font-weight: 700;
}

.checkoutUserEmail {
  color: color-mix(in srgb, var(--color-text) 62%, transparent);
  font-size: 12px;
  overflow-wrap: anywhere;
}

.selectedCheckoutUser {
  display: grid;
  gap: 2px;
  padding: 10px 12px;
  border: 1px solid color-mix(in srgb, var(--color-accent) 28%, transparent);
  border-radius: 6px;
  background: color-mix(in srgb, var(--color-accent) 8%, var(--color-background));
}

.selectedCheckoutUserLabel,
.selectedCheckoutUserEmail,
.noCheckoutUsers {
  color: color-mix(in srgb, var(--color-text) 62%, transparent);
  font-size: 12px;
}

.selectedCheckoutUserName {
  color: var(--color-text);
  font-size: 14px;
  font-weight: 700;
}

.noCheckoutUsers {
  padding: 10px;
}

.checkoutSummary {
  color: var(--color-text);
  font-size: 14px;
  font-weight: 500;
  line-height: 1.5;
}

.returnSummary {
  display: grid;
  gap: 4px;
  padding: 12px;
  border: 1px solid color-mix(in srgb, var(--color-warning) 28%, transparent);
  border-radius: 6px;
  background: color-mix(in srgb, var(--color-warning) 8%, var(--color-background));
}

.returnSummaryLabel,
.returnSummaryDate {
  color: color-mix(in srgb, var(--color-text) 62%, transparent);
  font-size: 12px;
}

.returnSummaryName {
  color: var(--color-text);
  font-size: 14px;
  font-weight: 700;
}

.checkoutConfirmActions {
  display: flex;
  gap: 8px;
  justify-content: stretch;
  margin-top: 14px;
}

.confirmCheckoutBtn {
  background: var(--color-accent);
  color: var(--color-background);
  flex: 1;
  min-width: 0;
}

.confirmCheckoutBtn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-accent) 80%, black);
}

.cancelCheckoutBtn {
  background: var(--color-background);
  color: var(--color-text);
  border: 1px solid color-mix(in srgb, var(--color-text) 20%, transparent);
  flex: 1;
  min-width: 0;
}

.cancelCheckoutBtn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-text) 14%, var(--color-background));
}

.empty {
  text-align: center;
  padding: 40px;
  color: color-mix(in srgb, var(--color-text) 60%, transparent);
  font-size: 18px;
}

.pagination {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: clamp(0.75rem, 3vw, 1rem);
  margin-top: clamp(1.5rem, 4vh, 2rem);
  padding: clamp(1rem, 3vh, 1.5rem);
  flex-wrap: wrap;
}

.paginationBtn {
  padding: clamp(0.5rem, 2vw, 0.75rem) clamp(0.75rem, 3vw, 1.25rem);
  border: 1px solid var(--color-accent);
  background: var(--color-background);
  color: var(--color-accent);
  border-radius: 6px;
  cursor: pointer;
  font-size: clamp(0.75rem, 2vw, 0.875rem);
  font-weight: 500;
  transition: all 0.2s;
  white-space: nowrap;
  min-width: clamp(60px, 15vw, 80px);
}

.paginationBtn:hover:not(:disabled) {
  background: var(--color-accent);
  color: var(--color-background);
}

.paginationBtn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pageInfo {
  font-size: 14px;
  color: var(--color-text);
  font-weight: 500;
}

.menuContainer {
  position: relative;
}

.menuBtn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 8px;
  border-radius: 4px;
  color: var(--color-text);
  font-size: 18px;
  font-weight: bold;
  line-height: 1;
  transition: background-color 0.2s;
}

.menuBtn:hover {
  background: color-mix(in srgb, var(--color-accent) 10%, transparent);
}

.dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  background: var(--color-background);
  border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
  border-radius: 4px;
  box-shadow: 0 4px 12px color-mix(in srgb, var(--color-accent) 30%, transparent);
  z-index: 100;
  min-width: 120px;
}

.dropdownItem {
  display: block;
  width: 100%;
  padding: 12px 16px;
  border: none;
  background: none;
  text-align: left;
  cursor: pointer;
  font-size: 14px;
  color: var(--color-text);
  transition: background-color 0.2s;
}

.dropdownItem:hover {
  background: color-mix(in srgb, var(--color-accent) 10%, transparent);
}

.deleteAction {
  color: var(--color-error);
}

.deleteAction:hover {
  background: color-mix(in srgb, var(--color-error) 10%, transparent);
}

.modalOverlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: var(--color-background);
  border-radius: 8px;
  width: 90%;
  max-width: 500px;
  max-height: 90vh;
  overflow: auto;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
}

.modalHeader {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
}

.modalTitle {
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.closeBtn {
  background: none;
  border: none;
  font-size: 24px;
  color: color-mix(in srgb, var(--color-text) 60%, transparent);
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.closeBtn:hover {
  background: color-mix(in srgb, var(--color-accent) 10%, transparent);
  color: var(--color-text);
}

.modalContent {
  padding: 24px;
}

@media (max-width: 40rem) {
  .container {
    width: min(100% - 24px, 1180px);
    padding: 16px 12px;
  }

  .itemCard {
    grid-template-columns: 1fr;
    grid-template-rows: auto auto;
    gap: 18px;
    padding: 20px;
  }

  .itemInfo {
    justify-self: stretch;
    padding-top: 28px;
  }

  .itemActions {
    justify-self: stretch;
    width: 100%;
    justify-content: flex-end;
  }

  .primaryActions {
    display: flex;
    gap: clamp(0.5rem, 1.5vw, 0.75rem);
  }

  .checkoutBtn,
  .returnBtn {
    min-width: 112px;
  }

  .checkoutOverlay {
    align-items: flex-end;
    padding: 16px;
  }

  .checkoutConfirm {
    width: 100%;
    max-height: min(78vh, 620px);
  }

  .pagination {
    flex-direction: column;
  }

  .paginationBtn {
    width: 100%;
    max-width: 200px;
  }

  .searchBar {
    grid-template-columns: 1fr;
    align-items: stretch;
  }

  .searchCount {
    justify-self: end;
  }
}
</style>
