<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useAppStore } from '../../stores/counter'
import type { CreateItemRequest, Item } from '../../types/api'
import { getErrorMessage } from '../../types/error'

interface Props {
  item: Item
}

const props = defineProps<Props>()
const emit = defineEmits<{
  close: []
}>()

const store = useAppStore()
const loading = ref(false)
const error = ref<string | null>(null)

const formData = ref({
  name: '',
  description: '',
  category: 'general' as 'general' | 'book' | 'laptop',
  author: undefined as string | undefined,
  isbn: undefined as string | undefined,
  macAddress: undefined as string | undefined,
})

const handleCategoryChange = () => {
  // Clear category-specific fields when category changes
  formData.value.author = undefined
  formData.value.isbn = undefined
  formData.value.macAddress = undefined
}

const handleSubmit = async () => {
  loading.value = true
  error.value = null

  try {
    const requestData: CreateItemRequest = (() => {
      switch (formData.value.category) {
        case 'book':
          return {
            category: 'book',
            name: formData.value.name,
            description: formData.value.description,
            author: formData.value.author || '',
            isbn: formData.value.isbn || '',
          }
        case 'laptop':
          return {
            category: 'laptop',
            name: formData.value.name,
            description: formData.value.description,
            mac_address: formData.value.macAddress || '',
          }
        default:
          return {
            category: 'general',
            name: formData.value.name,
            description: formData.value.description,
          }
      }
    })()

    await store.updateItem(props.item.id, requestData)
    emit('close')
  } catch (err: unknown) {
    error.value = getErrorMessage(err)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  // Initialize form with current item data
  formData.value = {
    name: props.item.name,
    description: props.item.description,
    category: props.item.category,
    author: props.item.category === 'book' ? props.item.author : undefined,
    isbn: props.item.category === 'book' ? props.item.isbn : undefined,
    macAddress: props.item.category === 'laptop' ? props.item.macAddress : undefined,
  }
})
</script>
<template>
  <div :class="$style.editForm">
    <form @submit.prevent="handleSubmit">
      <div :class="$style.formGroup">
        <label :class="$style.label">アイテム名</label>
        <input
          v-model="formData.name"
          type="text"
          :class="$style.input"
          required
          placeholder="アイテム名を入力"
        />
      </div>

      <div :class="$style.formGroup">
        <label :class="$style.label">説明</label>
        <textarea
          v-model="formData.description"
          :class="$style.textarea"
          rows="3"
          placeholder="アイテムの説明を入力"
        />
      </div>

      <div :class="$style.formGroup">
        <label :class="$style.label">カテゴリ</label>
        <select v-model="formData.category" :class="$style.select" @change="handleCategoryChange">
          <option value="general">一般</option>
          <option value="book">書籍</option>
          <option value="laptop">ノートPC</option>
        </select>
      </div>

      <!-- Book specific fields -->
      <div v-if="formData.category === 'book'" :class="$style.categoryFields">
        <div :class="$style.formGroup">
          <label :class="$style.label">著者</label>
          <input
            v-model="formData.author"
            type="text"
            :class="$style.input"
            placeholder="著者名を入力"
          />
        </div>

        <div :class="$style.formGroup">
          <label :class="$style.label">ISBN</label>
          <input
            v-model="formData.isbn"
            type="text"
            :class="$style.input"
            placeholder="ISBN番号を入力"
          />
        </div>
      </div>

      <!-- Laptop specific fields -->
      <div v-if="formData.category === 'laptop'" :class="$style.categoryFields">
        <div :class="$style.formGroup">
          <label :class="$style.label">MACアドレス</label>
          <input
            v-model="formData.macAddress"
            type="text"
            :class="$style.input"
            placeholder="MACアドレスを入力"
          />
        </div>
      </div>

      <div v-if="error" :class="$style.error">
        {{ error }}
      </div>

      <div :class="$style.actions">
        <button type="button" @click="$emit('close')" :class="[$style.btn, $style.cancelBtn]">
          キャンセル
        </button>
        <button type="submit" :class="[$style.btn, $style.submitBtn]" :disabled="loading">
          {{ loading ? '更新中...' : 'アイテムを更新' }}
        </button>
      </div>
    </form>
  </div>
</template>

<style module>
.editForm {
  padding: 0;
}

.formGroup {
  margin-bottom: 20px;
}

.label {
  display: block;
  font-weight: 500;
  color: var(--color-text);
  margin-bottom: 8px;
  font-size: 14px;
}

.input,
.textarea,
.select {
  width: 100%;
  padding: 12px;
  border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
  border-radius: 4px;
  font-size: 14px;
  background: var(--color-background);
  color: var(--color-text);
  transition: border-color 0.2s;
  box-sizing: border-box;
}

.input:focus,
.textarea:focus,
.select:focus {
  outline: none;
  border-color: var(--color-accent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-accent) 10%, transparent);
}

.textarea {
  resize: vertical;
  min-height: 80px;
}

.categoryFields {
  border-top: 1px solid color-mix(in srgb, var(--color-accent) 20%, transparent);
  padding-top: 20px;
  margin-top: 20px;
}

.error {
  background: color-mix(in srgb, var(--color-error) 10%, var(--color-background));
  color: var(--color-error);
  padding: 12px;
  border-radius: 4px;
  margin-bottom: 20px;
  font-size: 14px;
}

.actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  margin-top: 24px;
}

.btn {
  padding: 12px 24px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.cancelBtn {
  background: color-mix(in srgb, var(--color-text) 20%, transparent);
  color: var(--color-text);
}

.cancelBtn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-text) 30%, transparent);
}

.submitBtn {
  background: var(--color-accent);
  color: var(--color-background);
}

.submitBtn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-accent) 80%, black);
}
</style>
