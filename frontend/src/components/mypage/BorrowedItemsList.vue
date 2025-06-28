<script setup lang="ts">
import axios from "axios";
import { computed, onMounted, ref } from "vue";
import { useAppStore } from "../../stores/counter";
import type { Item } from "../../types/api";
import { getErrorMessage } from "../../types/error";

const store = useAppStore();
const borrowedItems = ref<Item[]>([]);
const loading = ref(false);
const returnLoading = ref(false);
const error = ref("");

const fetchBorrowedItems = async () => {
  loading.value = true;
  error.value = "";

  try {
    const token = localStorage.getItem("accessToken");
    const response = await axios.get(
      "https://procon-manager-item-manager-zcuq.shuttle.app/api/v1/users/me/checkouts",
      {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      },
    );

    // APIから返されるCheckoutのデータから、itemIdを使って詳細なアイテム情報を取得
    const checkouts = response.data.items;

    // 各チェックアウトのアイテム詳細を取得
    const itemPromises = checkouts.map(async (checkout: any) => {
      const itemResponse = await axios.get(
        `https://procon-manager-item-manager-zcuq.shuttle.app/api/v1/items/${checkout.itemId}`,
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        },
      );

      // アイテムにチェックアウト情報を追加
      return {
        ...itemResponse.data,
        checkout: {
          id: checkout.id,
          checkedOutBy: {
            id: store.currentUser?.id,
            name: store.currentUser?.name,
          },
          checkedOutAt: checkout.checkedOutAt,
        },
      };
    });

    borrowedItems.value = await Promise.all(itemPromises);
  } catch (err: unknown) {
    error.value = getErrorMessage(err);
  } finally {
    loading.value = false;
  }
};

const getItemTypeLabel = (category: string) => {
  switch (category) {
    case "general":
      return "一般";
    case "book":
      return "書籍";
    case "laptop":
      return "ノートPC";
    default:
      return "不明";
  }
};

const getItemDetails = (item: Item) => {
  switch (item.category) {
    case "book":
      return `著者: ${item.author}, ISBN: ${item.isbn}`;
    case "laptop":
      return `MAC: ${item.macAddress}`;
    default:
      return "";
  }
};

const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleDateString("ja-JP", {
    year: "numeric",
    month: "long",
    day: "numeric",
  });
};

const getDaysPassed = (dateString: string) => {
  const checkoutDate = new Date(dateString);
  const today = new Date();
  const diffTime = Math.abs(today.getTime() - checkoutDate.getTime());
  const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
  return diffDays;
};

const handleReturn = async (item: Item) => {
  if (!item.checkout) return;

  if (!confirm(`「${item.name}」を返却しますか？`)) return;

  returnLoading.value = true;

  try {
    await store.returnItem(item.id, item.checkout.id);
    // 返却後にリストを更新
    await fetchBorrowedItems();
    alert("返却が完了しました。");
  } catch (err: unknown) {
    error.value = getErrorMessage(err);
    alert(`返却に失敗しました: ${getErrorMessage(err)}`);
  } finally {
    returnLoading.value = false;
  }
};

onMounted(async () => {
  await store.getCurrentUser();
  await fetchBorrowedItems();
});
</script>

<template>
  <div :class="$style.container">
    <div v-if="loading" :class="$style.loading">
      読み込み中...
    </div>

    <div v-else-if="error" :class="$style.error">
      {{ error }}
    </div>

    <div v-else-if="borrowedItems.length === 0" :class="$style.empty">
      現在借用中の物品はありません
    </div>

    <div v-else :class="$style.itemsList">
      <div v-for="item in borrowedItems" :key="item.id" :class="$style.itemCard">
        <div :class="$style.itemInfo">
          <div :class="$style.itemHeader">
            <h3 :class="$style.itemName">{{ item.name }}</h3>
            <span :class="$style.itemType">{{ getItemTypeLabel(item.category) }}</span>
          </div>

          <p :class="$style.description">{{ item.description }}</p>

          <div v-if="getItemDetails(item)" :class="$style.details">
            {{ getItemDetails(item) }}
          </div>

          <div v-if="item.checkout" :class="$style.checkoutInfo">
            <div :class="$style.checkoutItem">
              <span :class="$style.checkoutLabel">借用日:</span>
              <span :class="$style.checkoutDate">{{ formatDate(item.checkout.checkedOutAt) }}</span>
            </div>
            <div :class="$style.checkoutItem">
              <span :class="$style.checkoutLabel">経過日数:</span>
              <span :class="$style.daysPassed">{{ getDaysPassed(item.checkout.checkedOutAt) }}日</span>
            </div>
          </div>
        </div>

        <div :class="$style.itemActions">
          <button @click="handleReturn(item)" :class="$style.returnBtn" :disabled="returnLoading">
            {{ returnLoading ? '返却中...' : '返却' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style module>
.container {
  width: 100%;
}

.loading {
  text-align: center;
  padding: 40px;
  color: color-mix(in srgb, var(--color-text) 60%, transparent);
}

.error {
  background: color-mix(in srgb, var(--color-error) 10%, var(--color-background));
  color: var(--color-error);
  padding: 12px;
  border-radius: 6px;
  margin-bottom: 20px;
}

.empty {
  text-align: center;
  padding: 40px;
  color: color-mix(in srgb, var(--color-text) 60%, transparent);
  font-size: 16px;
}

.itemsList {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.itemCard {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px;
  border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
  border-radius: 12px;
  background: var(--color-background);
  box-shadow: 0 2px 8px color-mix(in srgb, var(--color-accent) 15%, transparent);
  border-left: 4px solid var(--color-warning);
}

.itemInfo {
  flex: 1;
  margin-right: 24px;
}

.itemHeader {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.itemName {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text);
}

.itemType {
  background: color-mix(in srgb, var(--color-accent) 15%, var(--color-background));
  color: var(--color-accent);
  padding: 4px 8px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.description {
  color: color-mix(in srgb, var(--color-text) 70%, transparent);
  margin: 8px 0;
  line-height: 1.5;
  font-size: 14px;
}

.details {
  font-size: 14px;
  color: var(--color-text);
  margin: 8px 0;
  padding: 8px 12px;
  background: color-mix(in srgb, var(--color-accent) 8%, var(--color-background));
  border-radius: 6px;
  border-left: 3px solid var(--color-accent);
}

.checkoutInfo {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 12px;
  padding: 12px;
  background: color-mix(in srgb, var(--color-warning) 8%, var(--color-background));
  border-radius: 6px;
}

.checkoutItem {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
}

.checkoutLabel {
  font-weight: 500;
  color: var(--color-text);
  min-width: 80px;
}

.checkoutDate {
  color: color-mix(in srgb, var(--color-text) 80%, transparent);
}

.daysPassed {
  color: var(--color-warning);
  font-weight: 500;
}

.itemActions {
  display: flex;
  align-items: center;
}

.returnBtn {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  background: var(--color-warning);
  color: var(--color-background);
  transition: background-color 0.2s;
}

.returnBtn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-warning) 80%, black);
}

.returnBtn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@media (max-width: 768px) {
  .itemCard {
    flex-direction: column;
    align-items: stretch;
    gap: 16px;
  }

  .itemInfo {
    margin-right: 0;
  }

  .itemHeader {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }

  .checkoutInfo {
    margin-top: 8px;
  }

  .itemActions {
    justify-content: flex-end;
  }
}
</style>
