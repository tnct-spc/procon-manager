<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useAppStore } from '../../stores/counter'
import type { Item } from '../../types/api'
import type { ApiError } from '../../types/error'
import AddButton from '../ui/AddButton.vue'
import CreateItemForm from './CreateItemForm.vue'
import EditItemForm from './EditItemForm.vue'

const store = useAppStore()
const showCreateForm = ref(false)

onMounted(async () => {
  await store.fetchItems()
  await store.getCurrentUser()
})

const handleCheckout = async (item: Item) => {
  try {
    await store.checkoutItem(item.id)
  } catch (error: unknown) {
    console.error('チェックアウトエラー:', error)
    const apiError = error as ApiError
    if (apiError.response?.status === 404) {
      alert('サーバーに接続できません。バックエンドAPIが起動していることを確認してください。')
    } else if (apiError.response?.status === 409) {
      alert('このアイテムは既にチェックアウトされています。')
    } else {
      alert(`チェックアウトに失敗しました: ${apiError.message || 'サーバーエラー'}`)
    }
  }
}

const handleReturn = async (item: Item) => {
  if (item.checkout) {
    try {
      await store.returnItem(item.id, item.checkout.id)
    } catch (error: unknown) {
      console.error('返却エラー:', error)
      const apiError = error as ApiError
      if (apiError.response?.status === 404) {
        alert('サーバーに接続できません。バックエンドAPIが起動していることを確認してください。')
      } else {
        alert(`返却に失敗しました: ${apiError.message || 'サーバーエラー'}`)
      }
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
  showMenu.value = {
    ...showMenu.value,
    [itemId]: !showMenu.value[itemId],
  }
}

const editItem = (item: Item) => {
  editingItem.value = item
  showEditForm.value = true
  showMenu.value = {}
}

const deleteItem = async (itemId: string) => {
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
</script>

<template>
  <div :class="$style.container">
    <div :class="$style.header">
      <h1>アイテム管理</h1>
    </div>

    <div v-if="store.error" :class="$style.error">
      {{ store.error }}
    </div>

    <div v-if="store.loading" :class="$style.loading">読み込み中...</div>

    <div v-else :class="$style.itemList">
      <div
        v-for="item in store.items"
        :key="item.id"
        :class="[$style.itemCard, { [$style.checkedOut]: item.checkout }]"
      >
        <span :class="$style.itemType">{{ getItemTypeLabel(item) }}</span>
        <div :class="$style.itemInfo">
          <div :class="$style.itemHeader">
            <h3 :class="$style.itemName">{{ item.name }}</h3>
          </div>

          <p :class="$style.description">{{ item.description }}</p>

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
              :class="$style.checkoutBtn"
              :disabled="store.loading"
            >
              チェックアウト
            </button>

            <button
              v-else-if="
                store.currentUser &&
                (store.currentUser.role === 'Admin' ||
                  item.checkout.checkedOutBy.id === store.currentUser.id)
              "
              @click="handleReturn(item)"
              :class="$style.returnBtn"
              :disabled="store.loading"
            >
              返却
            </button>
          </div>

          <div :class="$style.menuContainer">
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

    <div v-if="store.items.length === 0 && !store.loading" :class="$style.empty">
      アイテムがありません
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
      @click="showCreateForm = true"
      :disabled="store.loading"
      label="新しいアイテムを追加"
    />
  </div>
</template>

<style module>
.container {
  padding: clamp(1rem, 3vw, 1.5rem);
  width: clamp(90%, 80vw, 80%);
  max-width: 100%;
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

.itemList {
  display: grid;
  gap: 24px;
}

.itemCard {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: clamp(1rem, 3vw, 2rem);
  padding: clamp(1.5rem, 4vw, 2rem);
  border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
  border-radius: 12px;
  background: var(--color-background);
  box-shadow: 0 4px 8px color-mix(in srgb, var(--color-accent) 20%, transparent);
  transition: all 0.2s;
  min-height: clamp(100px, 15vh, 120px);
  position: relative;
}

.itemCard:hover {
  border-color: var(--color-accent);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px color-mix(in srgb, var(--color-accent) 30%, transparent);
}

.checkedOut {
  border-left: 4px solid var(--color-error);
}

.itemInfo {
  justify-self: start;
  min-width: clamp(250px, 60%, 400px);
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
  top: clamp(0.75rem, 2vw, 1rem);
  right: clamp(0.75rem, 2vw, 1rem);
  background: color-mix(in srgb, var(--color-accent) 15%, var(--color-background));
  color: var(--color-accent);
  padding: clamp(0.25rem, 1vw, 0.5rem) clamp(0.5rem, 1.5vw, 0.75rem);
  border-radius: 12px;
  font-size: clamp(0.75rem, 2vw, 0.875rem);
  font-weight: 500;
  white-space: nowrap;
  z-index: 10;
}

.description {
  color: color-mix(in srgb, var(--color-text) 70%, transparent);
  margin: clamp(0.5rem, 2vw, 0.75rem) 0;
  line-height: 1.5;
  font-size: clamp(0.875rem, 2.5vw, 1rem);
}

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
  display: flex;
  align-items: center;
  gap: clamp(0.75rem, 2vw, 1rem);
  min-width: clamp(150px, 25vw, 200px);
  flex-wrap: wrap;
  justify-content: flex-end;
  justify-self: end;
}

.primaryActions {
  display: flex;
  gap: clamp(0.5rem, 1.5vw, 0.75rem);
  flex-wrap: wrap;
}

.checkoutBtn,
.returnBtn {
  padding: clamp(0.5rem, 2vw, 0.75rem) clamp(0.75rem, 3vw, 1rem);
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: clamp(0.75rem, 2vw, 0.875rem);
  transition: background-color 0.2s;
  white-space: nowrap;
  min-width: clamp(80px, 20vw, 120px);
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
.returnBtn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
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
  .itemCard {
    grid-template-columns: 1fr;
    grid-template-rows: auto auto;
  }

  .itemInfo {
    justify-self: stretch;
  }

  .itemActions {
    justify-self: stretch;
    justify-content: flex-end;
  }

  .primaryActions {
    display: flex;
    gap: clamp(0.5rem, 1.5vw, 0.75rem);
  }

  .checkoutBtn,
  .returnBtn {
    min-width: clamp(80px, 20vw, 120px);
  }

  .pagination {
    flex-direction: column;
  }

  .paginationBtn {
    width: 100%;
    max-width: 200px;
  }
}
</style>
