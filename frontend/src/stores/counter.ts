import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import api from '../services/api'
import type { CreateItemRequest, Item, PaginatedItemResponse, User } from '../types/api'
import { getErrorMessage } from '../types/error'

export const useAppStore = defineStore('app', () => {
  const items = ref<Item[]>([])
  const currentUser = ref<User | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Pagination state
  const currentPage = ref(1)
  const itemsPerPage = ref(10)
  const totalItems = ref(0)

  const fetchItems = async (page: number = 1) => {
    loading.value = true
    error.value = null
    try {
      const offset = (page - 1) * itemsPerPage.value
      const response = await api.get<PaginatedItemResponse>('/items', {
        params: { limit: itemsPerPage.value, offset },
      })
      items.value = response.data.items
      totalItems.value = response.data.total
      currentPage.value = page
      console.log('Fetched items:', items.value)
    } catch (err: unknown) {
      error.value = getErrorMessage(err)
      console.error('API error:', err)
    } finally {
      loading.value = false
    }
  }

  const createItem = async (itemData: CreateItemRequest) => {
    loading.value = true
    error.value = null
    try {
      console.log('Creating item with data:', itemData)
      await api.post('/items', itemData)
      await fetchItems(currentPage.value)
    } catch (err: unknown) {
      console.error('Create item error:', err)
      error.value = getErrorMessage(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  const checkoutItem = async (itemId: string) => {
    loading.value = true
    error.value = null
    try {
      await api.post(`/items/${itemId}/checkouts`)
      await fetchItems(currentPage.value)
    } catch (err: unknown) {
      error.value = getErrorMessage(err)
      console.error('チェックアウトエラー:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  const returnItem = async (itemId: string, checkoutId: string) => {
    loading.value = true
    error.value = null
    try {
      await api.put(`/items/${itemId}/checkouts/${checkoutId}/returned`)
      await fetchItems(currentPage.value)
    } catch (err: unknown) {
      error.value = getErrorMessage(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  const updateItem = async (itemId: string, itemData: CreateItemRequest) => {
    loading.value = true
    error.value = null
    try {
      await api.put(`/items/${itemId}`, itemData)
      await fetchItems(currentPage.value)
    } catch (err: unknown) {
      error.value = getErrorMessage(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  const deleteItem = async (itemId: string) => {
    loading.value = true
    error.value = null
    try {
      await api.delete(`/items/${itemId}`)
      await fetchItems(currentPage.value)
    } catch (err: unknown) {
      error.value = getErrorMessage(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  const getCurrentUser = async () => {
    try {
      const response = await api.get<User>('/users/me')
      currentUser.value = response.data
    } catch (err: unknown) {
      console.error('ユーザー情報の取得に失敗:', err)
    }
  }

  const totalPages = computed(() => Math.ceil(totalItems.value / itemsPerPage.value))

  return {
    items,
    currentUser,
    loading,
    error,
    currentPage,
    itemsPerPage,
    totalItems,
    totalPages,
    fetchItems,
    createItem,
    checkoutItem,
    returnItem,
    updateItem,
    deleteItem,
    getCurrentUser,
  }
})
