<script setup lang="ts">
import { ref, computed } from 'vue'
import { getErrorMessage } from '../../types/error'
import client from '../../services/api'

const emit = defineEmits<{
  success: []
}>()

const props = defineProps<{
  currentEmail: string
}>()

const formData = ref({
  email: props.currentEmail,
})

const loading = ref(false)
const error = ref('')
const success = ref(false)

const isValidEmail = computed(() => {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  return emailRegex.test(formData.value.email)
})

const isFormValid = computed(() => {
  return (
    formData.value.email.trim() !== '' &&
    isValidEmail.value &&
    formData.value.email !== props.currentEmail
  )
})

const handleSubmit = async () => {
  if (!isFormValid.value) return

  loading.value = true
  error.value = ''
  success.value = false

  try {
    const { error } = await client.PUT('/api/v1/users/me/email', {
      body: {
        email: formData.value.email.trim(),
      },
    })

    if (error) {
      throw new Error('Failed to change email')
    }

    success.value = true
    emit('success')

    setTimeout(() => {
      success.value = false
    }, 5000)
  } catch (err: unknown) {
    error.value = getErrorMessage(err)
  } finally {
    loading.value = false
  }
}

const handleReset = () => {
  formData.value.email = props.currentEmail
  error.value = ''
  success.value = false
}
</script>

<template>
  <div :class="$style.container">
    <form @submit.prevent="handleSubmit" :class="$style.form">
      <div v-if="error" :class="$style.error">
        {{ error }}
      </div>

      <div v-if="success" :class="$style.success">メールアドレスが正常に変更されました</div>

      <div :class="$style.field">
        <label for="email" :class="$style.label">メールアドレス *</label>
        <input
          id="email"
          v-model="formData.email"
          type="email"
          required
          :class="[$style.input, { [$style.inputError]: formData.email !== '' && !isValidEmail }]"
          placeholder="メールアドレスを入力"
          maxlength="255"
        />
        <div v-if="formData.email !== '' && !isValidEmail" :class="$style.fieldError">
          有効なメールアドレスを入力してください
        </div>
      </div>

      <div :class="$style.actions">
        <button type="button" :class="$style.resetBtn" @click="handleReset" :disabled="loading">
          リセット
        </button>
        <button type="submit" :class="$style.submitBtn" :disabled="loading || !isFormValid">
          {{ loading ? '変更中...' : 'メールアドレスを変更' }}
        </button>
      </div>
    </form>
  </div>
</template>

<style module>
.container {
  max-width: clamp(400px, 80vw, 500px);
  width: 100%;
}

.form {
  display: flex;
  flex-direction: column;
  gap: clamp(1rem, 3vw, 1.5rem);
}

.error {
  background: color-mix(in srgb, var(--color-error) 10%, var(--color-background));
  color: var(--color-error);
  padding: clamp(0.5rem, 2vw, 0.75rem);
  border-radius: clamp(0.375rem, 1vw, 0.5rem);
  font-size: clamp(0.75rem, 2vw, 0.875rem);
}

.success {
  background: color-mix(in srgb, var(--color-success) 10%, var(--color-background));
  color: var(--color-success);
  padding: clamp(0.5rem, 2vw, 0.75rem);
  border-radius: clamp(0.375rem, 1vw, 0.5rem);
  font-size: clamp(0.75rem, 2vw, 0.875rem);
}

.field {
  display: flex;
  flex-direction: column;
  gap: clamp(0.5rem, 1.5vw, 0.75rem);
}

.label {
  font-weight: 500;
  color: var(--color-text);
  font-size: clamp(0.875rem, 2.5vw, 1rem);
}

.input {
  width: 100%;
  padding: clamp(0.5rem, 2vw, 0.75rem);
  border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
  border-radius: clamp(0.375rem, 1vw, 0.5rem);
  font-size: clamp(0.875rem, 2.5vw, 1rem);
  background: var(--color-background);
  color: var(--color-text);
  transition: border-color 0.2s;
  box-sizing: border-box;
}

.input:focus {
  outline: none;
  border-color: var(--color-accent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-accent) 10%, transparent);
}

.inputError {
  border-color: var(--color-error);
}

.inputError:focus {
  border-color: var(--color-error);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-error) 10%, transparent);
}

.fieldError {
  color: var(--color-error);
  font-size: clamp(0.75rem, 2vw, 0.875rem);
  margin-top: clamp(0.25rem, 1vw, 0.375rem);
}

.actions {
  display: flex;
  gap: clamp(0.5rem, 2vw, 1rem);
  justify-content: center;
  margin-top: clamp(0.5rem, 2vw, 1rem);
}

.resetBtn {
  padding: clamp(0.5rem, 2vw, 0.75rem) clamp(1rem, 3vw, 1.5rem);
  border: 1px solid color-mix(in srgb, var(--color-accent) 50%, transparent);
  border-radius: clamp(0.375rem, 1vw, 0.5rem);
  cursor: pointer;
  font-size: clamp(0.875rem, 2.5vw, 1rem);
  font-weight: 500;
  background: var(--color-background);
  color: var(--color-accent);
  transition: all 0.2s;
  min-width: clamp(100px, 25vw, 140px);
}

.resetBtn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-accent) 10%, var(--color-background));
}

.resetBtn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.submitBtn {
  padding: clamp(0.5rem, 2vw, 0.75rem) clamp(1rem, 3vw, 1.5rem);
  border: none;
  border-radius: clamp(0.375rem, 1vw, 0.5rem);
  cursor: pointer;
  font-size: clamp(0.875rem, 2.5vw, 1rem);
  font-weight: 500;
  background: var(--color-accent);
  color: var(--color-background);
  transition: all 0.2s;
  min-width: clamp(120px, 30vw, 180px);
}

.submitBtn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-accent) 80%, black);
}

.submitBtn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
