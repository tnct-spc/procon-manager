<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { getErrorMessage } from '../../types/error'
import client from '../../services/api'

const email = ref('')
const password = ref('')
const showPassword = ref(false)
const errorMessage = ref('')
const isSubmitting = ref(false)

const router = useRouter()

const login = async () => {
  isSubmitting.value = true
  errorMessage.value = ''
  try {
    const { data, error } = await client.POST('/auth/login', {
      body: {
        email: email.value,
        password: password.value,
      },
    })

    if (error) {
      throw new Error('Invalid credentials')
    }

    if (!data) {
      throw new Error('No response data')
    }

    const { accessToken, userId } = data
    localStorage.setItem('accessToken', accessToken)
    localStorage.setItem('userId', userId)
    router.push('/dashboard')
  } catch (err: unknown) {
    errorMessage.value = getErrorMessage(err)
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <div class="container">
    <div class="card">
      <h1 class="heading">Login</h1>
      <p class="subtext">Procon Manager</p>

      <form @submit.prevent="login" class="form">
        <div v-if="errorMessage" class="alert">
          {{ errorMessage }}
        </div>

        <label class="form-label" for="email">mail</label>
        <input v-model="email" id="email" type="email" required class="input" />

        <label class="form-label" for="password">password</label>
        <div class="password-group">
          <input
            v-model="password"
            :type="showPassword ? 'text' : 'password'"
            id="password"
            required
            class="input"
          />
          <button type="button" class="toggle" @click="showPassword = !showPassword">
            {{ showPassword ? '🐈' : '🐈‍⬛' }}
          </button>
        </div>

        <button class="submit-button" :disabled="isSubmitting">
          {{ isSubmitting ? 'ログイン中...' : 'ログイン' }}
        </button>
      </form>
    </div>
  </div>
</template>

<style scoped>
.container {
  width: clamp(90%, 80vw, 400px);
  max-width: 90vw;
  display: flex;
  align-items: center;
  justify-content: center;
}

.card {
  background: var(--color-background);
  padding: clamp(1.5rem, 4vw, 2rem);
  border-radius: clamp(0.5rem, 2vw, 1rem);
  box-shadow: 0 0 20px color-mix(in srgb, var(--color-accent) 20%, transparent);
  width: 100%;
  max-height: 90vh;
  text-align: center;
  border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
}

.heading {
  font-size: clamp(1.5rem, 5vw, 2rem);
  margin-bottom: clamp(0.5rem, 2vw, 0.75rem);
  color: var(--color-text);
}

.subtext {
  font-size: clamp(0.875rem, 3vw, 1rem);
  color: color-mix(in srgb, var(--color-text) 70%, transparent);
  margin-bottom: clamp(1rem, 3vw, 1.5rem);
}

.form {
  display: flex;
  flex-direction: column;
  gap: clamp(0.75rem, 3vw, 1rem);
}

.alert {
  background: color-mix(in srgb, var(--color-error) 10%, var(--color-background));
  color: var(--color-error);
  padding: clamp(0.5rem, 2vw, 0.75rem);
  border-radius: clamp(0.375rem, 1vw, 0.5rem);
  font-size: clamp(0.75rem, 2vw, 0.875rem);
}

.form-label {
  font-weight: 600;
  text-align: left;
  color: var(--color-text);
  font-size: clamp(0.875rem, 2.5vw, 1rem);
}

.input {
  width: 100%;
  padding: clamp(0.5rem, 2vw, 0.75rem);
  border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
  border-radius: clamp(0.25rem, 1vw, 0.375rem);
  font-size: clamp(0.875rem, 2.5vw, 1rem);
  background: var(--color-background);
  color: var(--color-text);
  box-sizing: border-box;
}

.password-group {
  position: relative;
}

.toggle {
  position: absolute;
  top: 50%;
  right: clamp(0.5rem, 2vw, 0.75rem);
  transform: translateY(-50%);
  background: none;
  border: none;
  cursor: pointer;
  font-size: clamp(0.875rem, 2.5vw, 1rem);
  padding: clamp(0.25rem, 1vw, 0.375rem);
  border-radius: 4px;
}

.submit-button {
  padding: clamp(0.75rem, 3vw, 1rem);
  font-size: clamp(0.875rem, 2.5vw, 1rem);
  background-color: var(--color-accent);
  color: var(--color-background);
  border: none;
  border-radius: clamp(0.25rem, 1vw, 0.375rem);
  cursor: pointer;
  transition: background 0.2s ease-in-out;
}

.submit-button:hover:not(:disabled) {
  background-color: color-mix(in srgb, var(--color-accent) 80%, black);
}

.submit-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
