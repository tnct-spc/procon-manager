<script setup lang="ts">
import { onMounted, ref } from "vue";
import api from "../services/api";
import { useAppStore } from "../stores/counter";
import type { User } from "../types/api";
import { getErrorMessage } from "../types/error";

const store = useAppStore();
const users = ref<User[]>([]);
const error = ref<string | null>(null);
const isUpdating = ref(false);
const showAddUserModal = ref(false);
const isCreating = ref(false);
const addUserError = ref<string | null>(null);
const newUser = ref({
  name: "",
  email: "",
  password: "",
  role: "User" as "User" | "Admin",
});

const fetchUsers = async () => {
  try {
    store.loading = true;
    error.value = null;
    const response = await api.get("/users");
    users.value = response.data.items || response.data;
    console.log("ユーザー一覧:", users.value);
  } catch (err: unknown) {
    error.value = getErrorMessage(err);
    console.error("ユーザー一覧の取得に失敗:", err);
  } finally {
    store.loading = false;
  }
};

const promoteToAdmin = async (userId: string) => {
  if (!confirm("このユーザーをAdminに昇格させますか？")) return;

  try {
    isUpdating.value = true;
    await api.put(`/users/${userId}/role`, { role: "Admin" });
    await fetchUsers();
  } catch (err: unknown) {
    error.value = getErrorMessage(err);
    console.error("Admin昇格に失敗:", err);
  } finally {
    isUpdating.value = false;
  }
};

const demoteToUser = async (userId: string) => {
  if (!confirm("このAdminをUserに降格させますか？")) return;

  try {
    isUpdating.value = true;
    await api.put(`/users/${userId}/role`, { role: "User" });
    await fetchUsers();
  } catch (err: unknown) {
    error.value = getErrorMessage(err);
    console.error("User降格に失敗:", err);
  } finally {
    isUpdating.value = false;
  }
};

const deleteUser = async (userId: string) => {
  if (!confirm("このユーザーを削除しますか？この操作は取り消せません。"))
    return;

  try {
    isUpdating.value = true;
    await api.delete(`/users/${userId}`);
    await fetchUsers();
  } catch (err: unknown) {
    error.value = getErrorMessage(err);
    console.error("ユーザー削除に失敗:", err);
  } finally {
    isUpdating.value = false;
  }
};

const createUser = async () => {
  try {
    isCreating.value = true;
    addUserError.value = null;

    await api.post("/users", {
      name: newUser.value.name,
      email: newUser.value.email,
      password: newUser.value.password,
      role: newUser.value.role,
    });

    await fetchUsers();
    closeModal();
  } catch (err: unknown) {
    addUserError.value = getErrorMessage(err);
    console.error("ユーザー作成に失敗:", err);
  } finally {
    isCreating.value = false;
  }
};

const closeModal = () => {
  showAddUserModal.value = false;
  addUserError.value = null;
  newUser.value = {
    name: "",
    email: "",
    password: "",
    role: "User",
  };
};

onMounted(() => {
  fetchUsers();
});
</script>

<template>
  <div :class="$style.adminContainer">
    <h1 :class="$style.title">管理者操作</h1>

    <div :class="$style.section">
      <div :class="$style.sectionHeader">
        <h2 :class="$style.sectionTitle">ユーザー管理</h2>
        <button @click="showAddUserModal = true" :class="$style.addUserBtn" title="ユーザーを追加">
          +
        </button>
      </div>

      <div v-if="store.loading" :class="$style.loading">
        読み込み中...
      </div>

      <div v-else-if="error" :class="$style.error">
        {{ error }}
      </div>

      <div v-else :class="$style.userList">
        <div v-for="user in users" :key="user.id" :class="$style.userCard">
          <div :class="$style.userInfo">
            <div :class="$style.userName">{{ user.name }}</div>
            <div :class="$style.userEmail">{{ user.email }}</div>
            <div :class="$style.userRole">
              <span :class="$style.roleLabel">権限:</span>
              <span :class="[
                $style.roleBadge, 
                user.role === 'Admin' ? $style.adminBadge : $style.userBadge
              ]">
                {{ user.role }}
              </span>
            </div>
          </div>

          <div :class="$style.userActions">
            <button v-if="user.role === 'User'" @click="promoteToAdmin(user.id)"
              :class="[$style.actionBtn, $style.promoteBtn]" :disabled="isUpdating">
              Admin昇格
            </button>

            <button v-if="user.role === 'Admin' && user.id !== store.currentUser?.id" @click="demoteToUser(user.id)"
              :class="[$style.actionBtn, $style.demoteBtn]" :disabled="isUpdating">
              User降格
            </button>

            <button v-if="user.id !== store.currentUser?.id" @click="deleteUser(user.id)"
              :class="[$style.actionBtn, $style.deleteBtn]" :disabled="isUpdating">
              アカウント削除
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- ユーザー追加モーダル -->
    <div v-if="showAddUserModal" :class="$style.modalOverlay" @click="closeModal">
      <div :class="$style.modal" @click.stop>
        <div :class="$style.modalHeader">
          <h3 :class="$style.modalTitle">新しいユーザーを追加</h3>
          <button @click="closeModal" :class="$style.closeBtn">×</button>
        </div>

        <form @submit.prevent="createUser" :class="$style.addUserForm">
          <div :class="$style.formGroup">
            <label :class="$style.label">名前</label>
            <input v-model="newUser.name" type="text" :class="$style.input" required placeholder="ユーザー名を入力" />
          </div>

          <div :class="$style.formGroup">
            <label :class="$style.label">メールアドレス</label>
            <input v-model="newUser.email" type="email" :class="$style.input" required
              placeholder="example@email.com" />
          </div>

          <div :class="$style.formGroup">
            <label :class="$style.label">パスワード</label>
            <input v-model="newUser.password" type="password" :class="$style.input" required placeholder="パスワードを入力"
              minlength="6" />
          </div>

          <div :class="$style.formGroup">
            <label :class="$style.label">権限</label>
            <select v-model="newUser.role" :class="$style.select">
              <option value="User">User</option>
              <option value="Admin">Admin</option>
            </select>
          </div>

          <div v-if="addUserError" :class="$style.error">
            {{ addUserError }}
          </div>

          <div :class="$style.modalActions">
            <button type="button" @click="closeModal" :class="[$style.actionBtn, $style.cancelBtn]">
              キャンセル
            </button>
            <button type="submit" :class="[$style.actionBtn, $style.createBtn]" :disabled="isCreating">
              {{ isCreating ? '作成中...' : 'ユーザーを作成' }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<style module>
.adminContainer {
  max-width: 1200px;
  width: 100%;
  padding: 24px;
  margin: 0 auto;
}

.title {
  font-size: 32px;
  font-weight: 600;
  color: #2c3e50;
  margin-bottom: 32px;
  text-align: center;
}

.section {
  background: white;
  border-radius: 8px;
  padding: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.sectionTitle {
  font-size: 24px;
  font-weight: 600;
  color: #34495e;
  margin-bottom: 24px;
  border-bottom: 2px solid #3498db;
  padding-bottom: 8px;
}

.loading {
  text-align: center;
  padding: 32px;
  color: #7f8c8d;
  font-size: 16px;
}

.error {
  background: #fff5f5;
  color: #e53e3e;
  padding: 16px;
  border-radius: 4px;
  border: 1px solid #fed7d7;
  margin-bottom: 16px;
}

.userList {
  display: grid;
  gap: 16px;
}

.userCard {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  background: #f8f9fa;
  transition: all 0.2s;
}

.userCard:hover {
  border-color: #3498db;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.userInfo {
  flex: 1;
}

.userName {
  font-size: 18px;
  font-weight: 600;
  color: #2c3e50;
  margin-bottom: 4px;
}

.userEmail {
  color: #7f8c8d;
  font-size: 14px;
  margin-bottom: 8px;
}

.userRole {
  display: flex;
  align-items: center;
  gap: 8px;
}

.roleLabel {
  font-size: 14px;
  color: #7f8c8d;
}

.roleBadge {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
}

.adminBadge {
  background: #ffeaa7;
  color: #d63031;
}

.userBadge {
  background: #ddd6fe;
  color: #7c3aed;
}

.userActions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.actionBtn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
}

.actionBtn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.promoteBtn {
  background: #27ae60;
  color: white;
}

.promoteBtn:hover:not(:disabled) {
  background: #229954;
}

.demoteBtn {
  background: #f39c12;
  color: white;
}

.demoteBtn:hover:not(:disabled) {
  background: #e67e22;
}

.deleteBtn {
  background: #e74c3c;
  color: white;
}

.deleteBtn:hover:not(:disabled) {
  background: #c0392b;
}

.sectionHeader {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.sectionTitle {
  margin-bottom: 0;
}

.addUserBtn {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: #3498db;
  color: white;
  border: none;
  font-size: 24px;
  font-weight: bold;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(52, 152, 219, 0.3);
}

.addUserBtn:hover {
  background: #2980b9;
  transform: scale(1.05);
  box-shadow: 0 4px 12px rgba(52, 152, 219, 0.4);
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
  background: white;
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
  border-bottom: 1px solid #e2e8f0;
}

.modalTitle {
  font-size: 20px;
  font-weight: 600;
  color: #2c3e50;
  margin: 0;
}

.closeBtn {
  background: none;
  border: none;
  font-size: 24px;
  color: #7f8c8d;
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
  background: #f8f9fa;
  color: #2c3e50;
}

.addUserForm {
  padding: 24px;
}

.formGroup {
  margin-bottom: 20px;
}

.label {
  display: block;
  font-weight: 500;
  color: #34495e;
  margin-bottom: 8px;
  font-size: 14px;
}

.input,
.select {
  width: 100%;
  padding: 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
  transition: border-color 0.2s;
  box-sizing: border-box;
}

.input:focus,
.select:focus {
  outline: none;
  border-color: #3498db;
  box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1);
}

.modalActions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  margin-top: 24px;
}

.cancelBtn {
  background: #95a5a6;
  color: white;
}

.cancelBtn:hover:not(:disabled) {
  background: #7f8c8d;
}

.createBtn {
  background: #27ae60;
  color: white;
}

.createBtn:hover:not(:disabled) {
  background: #229954;
}
</style>
