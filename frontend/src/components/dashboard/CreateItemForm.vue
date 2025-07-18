<script setup lang="ts">
import { computed, ref } from 'vue'
import { useAppStore } from '../../stores/counter'
import type { CreateItemRequest } from '../../types/api'

const emit = defineEmits<{
  close: []
}>()

const store = useAppStore()

const formData = ref({
  category: 'general' as 'general' | 'book' | 'laptop',
  name: '',
  description: '',
  author: '',
  isbn: '',
  macAddress: '',
})

const isFormValid = computed(() => {
  const { category, name, description, author, isbn, macAddress } = formData.value

  if (!name || !description) return false

  if (category === 'book' && (!author || !isbn)) return false
  if (category === 'laptop' && !macAddress) return false

  return true
})

const handleSubmit = async () => {
  if (!isFormValid.value) return

  const { category, name, description, author, isbn, macAddress } = formData.value

  let itemData: CreateItemRequest

  switch (category) {
    case 'book':
      itemData = { category, name, description, author, isbn }
      break
    case 'laptop':
      itemData = { category, name, description, mac_address: macAddress }
      break
    default:
      itemData = { category: 'general', name, description }
  }

  try {
    await store.createItem(itemData)
    emit('close')
    resetForm()
  } catch (error) {
    console.error('アイテム作成エラー:', error)
  }
}

const resetForm = () => {
  formData.value = {
    category: 'general',
    name: '',
    description: '',
    author: '',
    isbn: '',
    macAddress: '',
  }
}

const handleCancel = () => {
  resetForm()
  emit('close')
}
</script>

<template>
  <div :class="$style.overlay" @click="handleCancel">
    <div :class="$style.modal" @click.stop>
      <div :class="$style.header">
        <h2>新しいアイテムを追加</h2>
        <button :class="$style.closeBtn" @click="handleCancel">×</button>
      </div>

      <form @submit.prevent="handleSubmit" :class="$style.form">
        <div :class="$style.field">
          <label for="category">カテゴリ</label>
          <select id="category" v-model="formData.category" :class="$style.select">
            <option value="general">一般</option>
            <option value="book">書籍</option>
            <option value="laptop">ノートPC</option>
          </select>
        </div>

        <div :class="$style.field">
          <label for="name">名前 *</label>
          <input id="name" v-model="formData.name" type="text" required :class="$style.input" />
        </div>

        <div :class="$style.field">
          <label for="description">説明 *</label>
          <textarea
            id="description"
            v-model="formData.description"
            required
            :class="$style.textarea"
            rows="3"
          ></textarea>
        </div>

        <div v-if="formData.category === 'book'" :class="$style.categoryFields">
          <div :class="$style.field">
            <label for="author">著者 *</label>
            <input
              id="author"
              v-model="formData.author"
              type="text"
              required
              :class="$style.input"
            />
          </div>

          <div :class="$style.field">
            <label for="isbn">ISBN *</label>
            <input id="isbn" v-model="formData.isbn" type="text" required :class="$style.input" />
          </div>
        </div>

        <div v-if="formData.category === 'laptop'" :class="$style.categoryFields">
          <div :class="$style.field">
            <label for="macAddress">MACアドレス *</label>
            <input
              id="macAddress"
              v-model="formData.macAddress"
              type="text"
              placeholder="00:00:00:00:00:00"
              required
              :class="$style.input"
            />
          </div>
        </div>

        <div v-if="store.error" :class="$style.error">
          {{ store.error }}
        </div>

        <div :class="$style.actions">
          <button type="button" @click="handleCancel" :class="$style.cancelBtn">キャンセル</button>
          <button type="submit" :class="$style.submitBtn" :disabled="!isFormValid || store.loading">
            {{ store.loading ? '作成中...' : '作成' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<style module>
.overlay {
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
  border-radius: clamp(0.5rem, 2vw, 1rem);
  width: clamp(90%, 90vw, 500px);
  max-width: 90vw;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 4px 20px color-mix(in srgb, var(--color-accent) 30%, transparent);
  border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: clamp(1rem, 3vw, 1.5rem);
  border-bottom: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
}

.header h2 {
  margin: 0;
  color: var(--color-text);
  font-size: clamp(1.125rem, 3vw, 1.25rem);
}

.closeBtn {
  background: none;
  border: none;
  font-size: clamp(1.25rem, 3vw, 1.5rem);
  cursor: pointer;
  color: color-mix(in srgb, var(--color-text) 60%, transparent);
  padding: 0;
  width: clamp(24px, 6vw, 30px);
  height: clamp(24px, 6vw, 30px);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.closeBtn:hover {
  color: var(--color-text);
  background: color-mix(in srgb, var(--color-accent) 10%, transparent);
}

.form {
  padding: clamp(1rem, 3vw, 1.5rem);
}

.field {
  margin-bottom: clamp(0.75rem, 2vw, 1rem);
}

.field label {
  display: block;
  margin-bottom: clamp(0.25rem, 1vw, 0.5rem);
  font-weight: 500;
  color: var(--color-text);
  font-size: clamp(0.875rem, 2.5vw, 1rem);
}

.input,
.select,
.textarea {
  width: 100%;
  padding: clamp(0.5rem, 2vw, 0.75rem);
  border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
  border-radius: 4px;
  font-size: clamp(0.875rem, 2.5vw, 1rem);
  box-sizing: border-box;
  background: var(--color-background);
  color: var(--color-text);
  transition: border-color 0.2s;
}

.input:focus,
.select:focus,
.textarea:focus {
  outline: none;
  border-color: var(--color-accent);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-accent) 20%, transparent);
}

.textarea {
  resize: vertical;
  min-height: 60px;
}

.categoryFields {
  border-top: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
  padding-top: 16px;
  margin-top: 16px;
}

.error {
  background: color-mix(in srgb, var(--color-error) 10%, var(--color-background));
  color: var(--color-error);
  padding: 12px;
  border-radius: 4px;
  margin-bottom: 16px;
  font-size: 14px;
}

.actions {
  display: flex;
  gap: clamp(0.5rem, 2vw, 0.75rem);
  justify-content: flex-end;
  margin-top: clamp(1rem, 3vw, 1.5rem);
  padding-top: clamp(0.75rem, 2vw, 1rem);
  border-top: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
  flex-wrap: wrap;
}

.cancelBtn,
.submitBtn {
  padding: clamp(0.5rem, 2vw, 0.75rem) clamp(0.75rem, 3vw, 1.25rem);
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: clamp(0.875rem, 2.5vw, 1rem);
  font-weight: 500;
  transition: background-color 0.2s;
  white-space: nowrap;
  min-width: clamp(80px, 20vw, 100px);
}

.cancelBtn {
  background: color-mix(in srgb, var(--color-text) 20%, transparent);
  color: var(--color-text);
}

.cancelBtn:hover {
  background: color-mix(in srgb, var(--color-text) 30%, transparent);
}

.submitBtn {
  background: var(--color-accent);
  color: var(--color-background);
}

.submitBtn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-accent) 80%, black);
}

.submitBtn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@container (max-width: 30rem) {
  .actions {
    flex-direction: column;
    align-items: stretch;
  }

  .cancelBtn,
  .submitBtn {
    width: 100%;
    min-width: auto;
  }
}
</style>
