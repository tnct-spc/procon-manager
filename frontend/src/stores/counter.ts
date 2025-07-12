import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import client from '../services/api'
import type { User, Item, CreateItemRequest } from '../types/api'
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
      const { data, error } = await client.GET('/api/v1/items', {
        params: {
          query: {
            limit: itemsPerPage.value,
            offset,
          },
        },
      })

      if (error || !data) {
        throw new Error('Failed to fetch items')
      }

      items.value = data.items
      totalItems.value = data.total
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
      const { error } = await client.POST('/api/v1/items', {
        body: itemData,
      })

      if (error) {
        throw new Error('Failed to create item')
      }

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
      const { error } = await client.POST('/api/v1/items/{item_id}/checkouts', {
        params: {
          path: { item_id: itemId },
        },
      })

      if (error) {
        throw new Error('Failed to checkout item')
      }

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
      const { error } = await client.PUT(
        '/api/v1/items/{item_id}/checkouts/{checkout_id}/returned',
        {
          params: {
            path: {
              item_id: itemId,
              checkout_id: checkoutId,
            },
          },
        },
      )

      if (error) {
        throw new Error('Failed to return item')
      }

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
      const { error } = await client.PUT('/api/v1/items/{item_id}', {
        params: {
          path: { item_id: itemId },
        },
        body: itemData,
      })

      if (error) {
        throw new Error('Failed to update item')
      }

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
      const { error } = await client.DELETE('/api/v1/items/{item_id}', {
        params: {
          path: { item_id: itemId },
        },
      })

      if (error) {
        throw new Error('Failed to delete item')
      }

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
      const { data, error } = await client.GET('/api/v1/users/me')

      if (error || !data) {
        console.error('ユーザー情報の取得に失敗:', error)
        return
      }

      currentUser.value = data
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
