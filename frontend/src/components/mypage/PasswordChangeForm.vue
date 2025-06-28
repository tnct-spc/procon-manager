<script setup lang="ts">
import axios from "axios";
import { computed, ref } from "vue";
import { getErrorMessage } from "../../types/error";

const emit = defineEmits<{
  success: [];
}>();

const formData = ref({
  currentPassword: "",
  newPassword: "",
  confirmPassword: "",
});

const showCurrentPassword = ref(false);
const showNewPassword = ref(false);
const showConfirmPassword = ref(false);
const loading = ref(false);
const error = ref("");
const success = ref(false);

const passwordMismatch = computed(() => {
  return (
    formData.value.confirmPassword !== "" &&
    formData.value.newPassword !== formData.value.confirmPassword
  );
});

const isFormValid = computed(() => {
  return (
    formData.value.currentPassword !== "" &&
    formData.value.newPassword !== "" &&
    formData.value.confirmPassword !== "" &&
    formData.value.newPassword === formData.value.confirmPassword &&
    formData.value.newPassword.length >= 6
  );
});

const handleSubmit = async () => {
  if (!isFormValid.value) return;

  loading.value = true;
  error.value = "";
  success.value = false;

  try {
    const token = localStorage.getItem("accessToken");
    await axios.put(
      "https://procon-manager-item-manager-zcuq.shuttle.app/api/v1/users/me/password",
      {
        currentPassword: formData.value.currentPassword,
        newPassword: formData.value.newPassword,
      },
      {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      },
    );

    success.value = true;
    formData.value = {
      currentPassword: "",
      newPassword: "",
      confirmPassword: "",
    };

    emit("success");

    setTimeout(() => {
      success.value = false;
    }, 5000);
  } catch (err: unknown) {
    error.value = getErrorMessage(err);
  } finally {
    loading.value = false;
  }
};
</script>

<template>
  <div :class="$style.container">
    <form @submit.prevent="handleSubmit" :class="$style.form">
      <div v-if="error" :class="$style.error">
        {{ error }}
      </div>

      <div v-if="success" :class="$style.success">
        パスワードが正常に変更されました
      </div>

      <div :class="$style.field">
        <label for="currentPassword" :class="$style.label">現在のパスワード *</label>
        <div :class="$style.passwordGroup">
          <input id="currentPassword" v-model="formData.currentPassword"
            :type="showCurrentPassword ? 'text' : 'password'" required :class="$style.input"
            placeholder="現在のパスワードを入力" />
          <button type="button" :class="$style.toggle" @click="showCurrentPassword = !showCurrentPassword">
            {{ showCurrentPassword ? '🐈' : '🐈‍⬛' }}
          </button>
        </div>
      </div>

      <div :class="$style.field">
        <label for="newPassword" :class="$style.label">新しいパスワード *</label>
        <div :class="$style.passwordGroup">
          <input id="newPassword" v-model="formData.newPassword" :type="showNewPassword ? 'text' : 'password'" required
            :class="$style.input" placeholder="新しいパスワードを入力" minlength="6" />
          <button type="button" :class="$style.toggle" @click="showNewPassword = !showNewPassword">
            {{ showNewPassword ? '🐈' : '🐈‍⬛' }}
          </button>
        </div>
      </div>

      <div :class="$style.field">
        <label for="confirmPassword" :class="$style.label">新しいパスワード（確認） *</label>
        <div :class="$style.passwordGroup">
          <input id="confirmPassword" v-model="formData.confirmPassword"
            :type="showConfirmPassword ? 'text' : 'password'" required
            :class="[$style.input, { [$style.inputError]: passwordMismatch }]" placeholder="新しいパスワードを再入力"
            minlength="6" />
          <button type="button" :class="$style.toggle" @click="showConfirmPassword = !showConfirmPassword">
            {{ showConfirmPassword ? '🐈' : '🐈‍⬛' }}
          </button>
        </div>
        <div v-if="passwordMismatch" :class="$style.fieldError">
          パスワードが一致しません
        </div>
      </div>

      <div :class="$style.actions">
        <button type="submit" :class="$style.submitBtn" :disabled="loading || !isFormValid">
          {{ loading ? '変更中...' : 'パスワードを変更' }}
        </button>
      </div>
    </form>
  </div>
</template>

<style module>
.container {
  max-width: 500px;
}

.form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.error {
  background: color-mix(in srgb, var(--color-error) 10%, var(--color-background));
  color: var(--color-error);
  padding: 12px;
  border-radius: 6px;
  font-size: 14px;
}

.success {
  background: color-mix(in srgb, var(--color-success) 10%, var(--color-background));
  color: var(--color-success);
  padding: 12px;
  border-radius: 6px;
  font-size: 14px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.label {
  font-weight: 500;
  color: var(--color-text);
  font-size: 14px;
}

.passwordGroup {
  position: relative;
}

.input {
  width: 100%;
  padding: 12px;
  border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
  border-radius: 6px;
  font-size: 14px;
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

.toggle {
  position: absolute;
  top: 50%;
  right: 12px;
  transform: translateY(-50%);
  background: none;
  border: none;
  cursor: pointer;
  font-size: 16px;
  padding: 4px;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.toggle:hover {
  background: color-mix(in srgb, var(--color-accent) 10%, transparent);
}

.fieldError {
  color: var(--color-error);
  font-size: 12px;
  margin-top: 4px;
}

.actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 8px;
}

.submitBtn {
  padding: 12px 24px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  background: var(--color-accent);
  color: var(--color-background);
  transition: all 0.2s;
}

.submitBtn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-accent) 80%, black);
}

.submitBtn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
