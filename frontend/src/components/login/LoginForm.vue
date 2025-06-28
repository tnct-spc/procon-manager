<script setup lang="ts">
import axios from "axios";
import { ref } from "vue";
import { useRouter } from "vue-router";
import type { LoginResponse } from "../../types/api";
import { getErrorMessage } from "../../types/error";

const email = ref("");
const password = ref("");
const showPassword = ref(false);
const errorMessage = ref("");
const isSubmitting = ref(false);

const router = useRouter();

const login = async () => {
  isSubmitting.value = true;
  errorMessage.value = "";
  try {
    const res = await axios.post<LoginResponse>(
      "https://procon-manager-item-manager-zcuq.shuttle.app/auth/login",
      { email: email.value, password: password.value },
    );
    const { accessToken, userId } = res.data;
    localStorage.setItem("accessToken", accessToken);
    localStorage.setItem("userId", userId);
    axios.defaults.headers.common["Authorization"] = `Bearer ${accessToken}`;
    router.push("/dashboard");
  } catch (err: unknown) {
    errorMessage.value = getErrorMessage(err);
  } finally {
    isSubmitting.value = false;
  }
};
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
          <input v-model="password" :type="showPassword ? 'text' : 'password'" id="password" required class="input" />
          <button type="button" class="toggle" @click="showPassword = !showPassword">
            {{ showPassword ? '🐈' : '🐈‍⬛'}}
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
  width: 80%;
  max-width: 400px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.card {
  background: var(--color-background);
  padding: 2rem;
  border-radius: 1rem;
  box-shadow: 0 0 20px color-mix(in srgb, var(--color-accent) 20%, transparent);
  width: 100%;
  /* max-width: 400px; */
  max-height: 90vh;
  text-align: center;
  border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
}

.heading {
  font-size: 2rem;
  margin-bottom: 0.5rem;
  color: var(--color-text);
}

.subtext {
  font-size: 1rem;
  color: color-mix(in srgb, var(--color-text) 70%, transparent);
  margin-bottom: 1.5rem;
}

.form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.alert {
  background: color-mix(in srgb, var(--color-error) 10%, var(--color-background));
  color: var(--color-error);
  padding: 0.75rem;
  border-radius: 0.5rem;
  font-size: 0.875rem;
}

.form-label {
  font-weight: 600;
  text-align: left;
  color: var(--color-text);
}

.input {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
  border-radius: 0.375rem;
  font-size: 1rem;
  background: var(--color-background);
  color: var(--color-text);
}

.password-group {
  position: relative;
}

.toggle {
  position: absolute;
  top: 50%;
  right: 0.75rem;
  transform: translateY(-50%);
  background: none;
  border: none;
  cursor: pointer;
  font-size: 1rem;
}

.submit-button {
  padding: 0.75rem;
  font-size: 1rem;
  background-color: var(--color-accent);
  color: var(--color-background);
  border: none;
  border-radius: 0.375rem;
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
